//! This file is responsible for transferring data between the server and the node

use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Instant, Timer};

use shared::telemetry::NodeTelemetry;
use shared::packets::{Packet, PacketPayload};

use defmt::{error, info};

use crate::NODE_CONFIG;
use crate::network_status::{NETWORK_STATUS, NetworkStatus};

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
    S: PacketSender, <S as PacketSender>::Error: defmt::Format
{
    let tx = NETWORK_STATUS.sender();
    let mut rx = NETWORK_STATUS.receiver().unwrap();
    loop {
        let telemetry = TELEMETRY_CHANNEL.receive().await;
        let packet = create_packet(PacketPayload::Telemetry(telemetry)).await;
        info!("Telemetry packet created: {}", packet);

        let mut buf = [0u8; 256];
        let res = packet.serialize(&mut buf);

        match res {
            Ok(n) => {
                let success = sender.send(&buf[..n]).await;
                match success {
                    Ok(_) => {
                        let state = rx.get().await;
                        if state != NetworkStatus::Connected {
                            tx.send(NetworkStatus::Connected);
                        }
                    },
                    Err(e) => {
                        error!("Failed to send packet: {}", e);
                        tx.send(NetworkStatus::HostNotFound);
                    }
                }
            },
            Err(e) => {
                error!("Failed to serialize: {}", packet);
            }
        }

        Timer::after_millis(200).await;
    }
}
