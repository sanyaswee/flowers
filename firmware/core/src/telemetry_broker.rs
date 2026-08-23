//! Gathering all telemetry and sending it to the server

use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_time::Timer;

use shared::telemetry::NodeTelemetry;

use defmt::info;
use shared::packets::PacketPayload;
use crate::network::{create_packet, TELEMETRY_CHANNEL};

/// Shared telemetry mutex
pub static TELEMETRY: Mutex<ThreadModeRawMutex, NodeTelemetry> = Mutex::new(NodeTelemetry::new());

/// Main broker task
#[embassy_executor::task]
pub async fn gather() {
    loop {
        // Needed to drop the lock
        let t = {
            let t = TELEMETRY.lock().await;
            t.clone()
        };
        info!("Telemetry gathered: {}", t);
        let packet = create_packet(PacketPayload::Telemetry(t)).await;
        TELEMETRY_CHANNEL.send(packet).await;
        Timer::after_secs(5).await;
    }
}