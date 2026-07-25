//! This file is responsible for transferring data between the server and the node

use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Instant, Timer};

use shared::telemetry::NodeTelemetry;
use shared::packets::TelemetryPacket;

use defmt::info;

use crate::NODE_CONFIG;

/// The channel for receiving the telemetry from telemetry_broker
pub static TELEMETRY_CHANNEL: Channel<ThreadModeRawMutex, NodeTelemetry, 64> = Channel::new();

/// Function to create a packet from NodeTelemetry
async fn create_packet(telemetry: NodeTelemetry) -> TelemetryPacket {
    let id = NODE_CONFIG.get().await.node_id;
    let uptime = Instant::now().as_millis();

    TelemetryPacket::new(id, uptime, telemetry)
}

/// The task to send the telemetry to the server
#[embassy_executor::task]
pub async fn telemetry_sender() {
    loop {
        let telemetry = TELEMETRY_CHANNEL.receive().await;
        let packet = create_packet(telemetry).await;
        info!("Telemetry packet created: {}", packet);
        Timer::after_millis(200).await;
    }
}
