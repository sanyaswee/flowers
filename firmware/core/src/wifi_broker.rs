//! This file is responsible for transferring data between the server and the node

use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Instant, Timer};

use shared::telemetry::NodeTelemetry;
use shared::packets::{Packet, PacketPayload};

use defmt::{error, info};

use crate::NODE_CONFIG;

/// The channel for receiving the telemetry from telemetry_broker
pub static TELEMETRY_CHANNEL: Channel<ThreadModeRawMutex, NodeTelemetry, 16> = Channel::new();

/// This trait should be implemented in the node specific crate
pub trait PacketSender {
    type Error;

    async fn send(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;
}

/// Function to create a packet from NodeTelemetry
async fn create_packet(payload: PacketPayload) -> Packet {
    let id = NODE_CONFIG.get().await.node_id;
    let uptime = Instant::now().as_millis();

    Packet::new(id, uptime, payload)
}

/// The task to send the telemetry to the server
pub async fn telemetry_sender<S>(sender: &mut S)
where
    S: PacketSender,
{
    loop {
        let telemetry = TELEMETRY_CHANNEL.receive().await;
        let packet = create_packet(PacketPayload::Telemetry(telemetry)).await;
        info!("Telemetry packet created: {}", packet);

        // TODO send

        Timer::after_millis(200).await;
    }
}
