//! Server-side auto watering
//!
//! Nodes know nothing about auto watering. This task wakes up periodically, looks at every
//! channel that has it enabled and, when the time has come, sends `Water` packet.
//!
//! Only the interval based type is implemented. The threshold based one needs a proper moisture
//! calibration first, so such channels are ignored for now.

use std::sync::Arc;
use std::time::Duration as StdDuration;

use chrono::{Duration, NaiveDateTime, Utc};
use rumqttc::AsyncClient;
use sqlx::SqlitePool;
use tokio::time::{self, MissedTickBehavior};

use crate::db::entries::{AutoWateringType, ChannelEntry, NodeEntry, NodeTelemetryEntry};
use crate::mqtt::router::water_plant;

/// How often channels are checked
const TICK: StdDuration = StdDuration::from_secs(30);

/// Time to wait before the next watering is allowed.
const PUMP_MARGIN_S: i64 = 10;

/// Start the auto watering task
pub fn spawn(pool: SqlitePool, client: Arc<AsyncClient>) {
    tokio::spawn(async move {
        let mut ticker = time::interval(TICK);
        // Do not fire a burst of catch-up ticks
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
        // The first tick ticks immediately, so we skip it
        ticker.tick().await;

        println!(
            "Auto watering task started (checking every {}s)",
            TICK.as_secs()
        );
        loop {
            ticker.tick().await;
            if let Err(e) = tick(&pool, &client).await {
                eprintln!("Auto watering: DB error: {e}");
            }
        }
    });
}

/// Iterate over all channels
async fn tick(pool: &SqlitePool, client: &Arc<AsyncClient>) -> Result<(), sqlx::Error> {
    let now = Utc::now().naive_utc();

    for channel in ChannelEntry::get_all(pool).await? {
        if !channel.enabled || !channel.auto_watering_enable {
            continue;
        }

        // TODO: threshold based watering, after calibration patches
        if channel.auto_watering_type != AutoWateringType::IntervalBased {
            continue;
        }
        if !is_due(
            channel.last_watered,
            now,
            channel.auto_watering_interval,
            channel.watering_time,
        ) {
            continue;
        }

        // Broken channel should not stop the others
        let name = format!("{}/{}", channel.node_id, channel.channel_id);
        if let Err(e) = water_if_possible(pool, client, channel, now).await {
            eprintln!("Auto watering: DB error for {name}: {e}");
        }
    }

    Ok(())
}

/// Send the watering request for a channel that is due, unless something blocks it
/// If skipped, the channel stays due and is retried on the next tick
async fn water_if_possible(
    pool: &SqlitePool,
    client: &Arc<AsyncClient>,
    mut channel: ChannelEntry,
    now: NaiveDateTime,
) -> Result<(), sqlx::Error> {
    let name = if channel.verbose_name.is_some() {
        format!(
            "{} [{}/{}]",
            channel.verbose_name.clone().unwrap(),
            channel.node_id,
            channel.channel_id
        )
    } else {
        format!("[{}/{}]", channel.node_id, channel.channel_id)
    };

    let Some(node) = NodeEntry::from_node_id(pool, &channel.node_id).await? else {
        return Ok(());
    };

    if channel.channel_id >= node.n_channels {
        eprintln!("Auto watering: {name} does not exist on the node");
        return Ok(());
    }

    // Commands are fire-and-forget, so for an unreachable node the request would be lost
    // and we would wrongly believe that the plant was watered
    if !is_online(node.last_active, now, node.telemetry_report_freq) {
        println!("Auto watering: {name} is due, but the node is offline");
        return Ok(());
    }

    // Do not run the pump dry
    if node.water_tank {
        let latest =
            NodeTelemetryEntry::from_node_id(pool, &node.node_id, None, None, Some(1)).await?;
        if latest.first().and_then(|t| t.water_tank_has_water) == Some(false) {
            println!("Auto watering: {name} is due, but the water tank is empty");
            return Ok(());
        }
    }

    println!("Auto watering: watering {name}");
    if water_plant(client.clone(), node, channel.channel_id).await {
        channel.mark_watered(pool).await?;
    }

    Ok(())
}

/// Check if it's time to water the channel
fn is_due(
    last_watered: Option<NaiveDateTime>,
    now: NaiveDateTime,
    interval_min: u16,
    watering_time_s: u16,
) -> bool {
    let Some(last) = last_watered else {
        return true;
    };

    let interval = Duration::minutes(i64::from(interval_min));
    let pump_time = Duration::seconds(i64::from(watering_time_s) + PUMP_MARGIN_S);

    now - last >= interval.max(pump_time)
}

/// Same rule as `nodeStatus` in the web UI
fn is_online(last_active: NaiveDateTime, now: NaiveDateTime, report_freq_s: u16) -> bool {
    now - last_active <= Duration::seconds(i64::from(report_freq_s) * 3 + 15)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(secs: i64) -> NaiveDateTime {
        chrono::DateTime::from_timestamp(1_700_000_000 + secs, 0)
            .unwrap()
            .naive_utc()
    }

    #[test]
    fn never_watered_is_due() {
        assert!(is_due(None, at(0), 60, 5));
    }

    #[test]
    fn due_only_after_the_interval() {
        // every 60 minutes
        assert!(!is_due(Some(at(0)), at(59 * 60 + 59), 60, 5));
        assert!(is_due(Some(at(0)), at(60 * 60), 60, 5));
        assert!(is_due(Some(at(0)), at(3 * 60 * 60), 60, 5));
    }

    #[test]
    fn requests_never_overlap_with_the_pump() {
        // 1 minute interval, but the pump runs for 120s: wait for it to finish (+ margin)
        assert!(!is_due(Some(at(0)), at(60), 1, 120));
        assert!(!is_due(Some(at(0)), at(129), 1, 120));
        assert!(is_due(Some(at(0)), at(130), 1, 120));
    }

    #[test]
    fn clock_set_back_is_not_due() {
        assert!(!is_due(Some(at(1000)), at(0), 1, 5));
    }

    #[test]
    fn online_window_matches_the_web_ui() {
        // report every 10s: online for up to 3 * 10 + 15 = 45s
        assert!(is_online(at(0), at(45), 10));
        assert!(!is_online(at(0), at(46), 10));
    }
}
