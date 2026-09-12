//! Handling of incoming MQTT messages

use defmt::{error, info};

use shared::node_settings::NodeSettings;
use shared::packets::{Packet, PacketPayload};

use crate::settings::OVERRIDE_SIG;


/// Match an incoming topic to its handler
pub async fn dispatch(topic: &str, payload: &[u8]) {
    info!("Incoming packet received on: {}", topic);
    match Packet::deserialize(payload) {
        Ok(packet) => {
            match packet.payload {
                PacketPayload::ServerBoot => handle_server_boot().await,
                PacketPayload::SettingsOverride(new) => handle_settings_override(new).await,
                unknown => error!("Unknown packet received: {}", unknown),
            }
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