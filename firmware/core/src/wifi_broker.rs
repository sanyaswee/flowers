//! This file is responsible for transferring data between the server and the node

use embassy_sync::blocking_mutex::raw::{RawMutex, ThreadModeRawMutex};
use embassy_sync::mutex::Mutex;
use embassy_sync::channel::Channel;
use embassy_time::{Instant, Timer};

use shared::telemetry::NodeTelemetry;
use shared::packets::{Packet, PacketPayload};

use defmt::{error, info};

use crate::NODE_CONFIG;
use crate::network_manager::{NETWORK_STATUS, NetworkStatus};

/// The channel for receiving the telemetry from telemetry_broker
pub static TELEMETRY_CHANNEL: Channel<ThreadModeRawMutex, NodeTelemetry, 16> = Channel::new();

/// This trait should be implemented in the node specific crate
pub trait PacketSender {
    type Error;

    /// Send the packet to the server
    async fn send(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;

    /// (Re)connect to the server
    async fn reconnect(&mut self) -> Result<(), Self::Error>;
}

/// Function to create a packet from NodeTelemetry
async fn create_packet(payload: PacketPayload) -> Packet {
    let id = NODE_CONFIG.get().await.node_id;
    let uptime = Instant::now().as_millis();

    Packet::new(id, uptime, payload)
}

/// The task to send the telemetry to the server
pub async fn telemetry_sender<M, S>(sender: &Mutex<M, S>)
where
    M: RawMutex,
    S: PacketSender, <S as PacketSender>::Error: defmt::Format
{
    let tx = NETWORK_STATUS.sender();
    let mut rx = NETWORK_STATUS.receiver().unwrap();

    loop {
        let telemetry = TELEMETRY_CHANNEL.receive().await;
        let packet = create_packet(PacketPayload::Telemetry(telemetry)).await;
        info!("Telemetry packet created: {}", packet);

        let mut buf = [0u8; 256];
        match packet.serialize(&mut buf) {
            Ok(n) => {
                let result = sender.lock().await.send(&buf[..n]).await;
                match result {
                    Ok(()) => {
                        if rx.get().await != NetworkStatus::Connected {
                            tx.send(NetworkStatus::Connected);
                        }
                    }
                    Err(e) => {
                        error!("Failed to send packet: {}", e);
                        tx.send(NetworkStatus::HostNotFound);
                    }
                }
            }
            Err(_) => {
                error!("Failed to serialize packet");
            }
        }

        Timer::after_millis(200).await;
    }
}
