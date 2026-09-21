//! Handling of incoming MQTT messages

use defmt::{error, info};

use heapless::String;

use minimq::TopicFilter;

use shared::mqtt_convention;
use shared::node_settings::NodeSettings;
use shared::packets::{Packet, PacketPayload};

use crate::NODE_CONFIG;
use crate::plant_channel::WATERING_SIGNALS;
use crate::settings::OVERRIDE_SIG;

/// Number of topics we are subscribed to
pub(crate) const TOPICS_NUM: usize = 3;

/// Return topics array to subscribe to
/// Looks a bit overcomplicated because I gave up fighting with Rust lifetimes
pub async fn topics<'a>(bufs: &'a mut [String<64>; TOPICS_NUM]) -> [TopicFilter<'a>; TOPICS_NUM] {
    for buf in bufs.iter_mut() {
        buf.clear();
    }

    let node_id = NODE_CONFIG.get().await.node_id.as_str();

    mqtt_convention::server_boot(&mut bufs[0]);
    mqtt_convention::settings_override(&mut bufs[1], node_id);
    mqtt_convention::water(&mut bufs[2], node_id);

    [
        TopicFilter::new(bufs[0].as_str()),
        TopicFilter::new(bufs[1].as_str()),
        TopicFilter::new(bufs[2].as_str()),
    ]
}

/// Match an incoming topic to its handler
pub async fn dispatch(topic: &str, payload: &[u8]) {
    info!("Incoming packet received on: {}", topic);
    match Packet::deserialize(payload) {
        Ok(packet) => match packet.payload {
            PacketPayload::ServerBoot => handle_server_boot().await,
            PacketPayload::SettingsOverride(new) => handle_settings_override(new).await,
            PacketPayload::Water(channel) => handle_water(channel).await,
            unknown => error!("Unknown packet received: {}", unknown),
        },
        Err(e) => {
            error!("Failed to deserialize incoming packet: {}", e)
        }
    }
}

/// Server boot handler
async fn handle_server_boot() {
    super::push_boot().await;
}

/// Settings override handler
async fn handle_settings_override(new: NodeSettings) {
    OVERRIDE_SIG.signal(new)
}

/// Water command handler
async fn handle_water(channel: u8) {
    WATERING_SIGNALS[channel as usize].signal(());
}
