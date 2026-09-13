//! This module contains hardware-generic network traits and tasks

mod router;
pub mod status;
pub(crate) mod priority;

use defmt::{error, info};

use embassy_futures::select::{select, Either};

use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::priority_channel::{PriorityChannel, Max};

use embassy_time::{Instant, Timer};

use embedded_io_async::{Read, Write};

use heapless::String;

use minimq::{Buffers, ConfigBuilder, Session, QoS, Publication, TopicFilter};

use shared::mqtt_convention;
use shared::packets::{Packet, PacketPayload};

use crate::settings;
use crate::NODE_CONFIG;

use status::{NetworkStatus, NETWORK_STATUS};
use priority::PriorityPacketWrapper;

/// The channel for receiving the telemetry from telemetry_broker
pub static PACKET_CHANNEL: PriorityChannel<ThreadModeRawMutex, PriorityPacketWrapper, Max, { settings::TELEMETRY_CHANNEL_SIZE  }> = PriorityChannel::new();

/// Should be implemented in the node specific crate
pub trait TcpProvider {
    type Stream: Read + Write;
    type Error: defmt::Format;

    /// Establish a raw TCP connection to the broker
    async fn connect(&mut self) -> Result<Self::Stream, Self::Error>;
}

/// Function to create a packet from generic payload
pub async fn create_packet(payload: PacketPayload) -> Packet {
    let uptime = Instant::now().as_millis();

    Packet::new(uptime, payload)
}

/// Push a node boot packet with config into packet channel
pub async fn push_boot() {
    let config = NODE_CONFIG.get().await;
    let boot_packet = create_packet(PacketPayload::NodeBoot(config.clone())).await;
    PACKET_CHANNEL.send(PriorityPacketWrapper(boot_packet)).await;
}

/// The MQTT task
pub async fn mqtt_network_task<T: TcpProvider>(mut tcp: T, client_id: &str) {
    let tx = NETWORK_STATUS.sender();

    push_boot().await;

    // Allocate buffers
    let mut rx_buf = [0u8; 256];
    let mut tx_buf = [0u8; 768];
    let buffers = Buffers::new(&mut rx_buf, &mut tx_buf);

    let config = ConfigBuilder::new(buffers)
        .client_id(client_id).unwrap()
        .keepalive_interval(60);
    let mut session = Session::new(config);

    info!("Packet size: {} bytes", size_of::<Packet>());

    loop {
        tx.send(NetworkStatus::HostNotFound);

        // Raw TCP stream
        let stream = match tcp.connect().await {
            Ok(s) => s,
            Err(_) => {
                Timer::after_secs(2).await;
                continue;
            }
        };

        // MQTT Handshake
        let mut conn = match session.connect(stream).await {
            Ok(c) => c,
            Err(_) => continue,
        };

        tx.send(NetworkStatus::Connected);

        // Subscribe to topics
        // Looks a bit overcomplicated, but I want to keep convention similar between sent and received topics
        let topics = [
            TopicFilter::new({
                let mut t: String<64> = String::new();
                mqtt_convention::server_boot(&mut t);
                t.as_str()
            }),
            TopicFilter::new({
                let mut t: String<64> = String::new();
                let id = NODE_CONFIG.get().await.node_id.clone();
                mqtt_convention::settings_override(&mut t, id.as_str());
                t.as_str()
            })
        ];
        if conn.subscribe(&topics, &[]).await.is_err() {
            continue; // reconnect
        }

        // Polling and publishing
        loop {
            match select(conn.poll(), PACKET_CHANNEL.receive()).await {
                // Connection or protocol error, drop and reconnect
                Either::First(Err(_)) => break,
                // Idle poll success
                Either::First(Ok(None)) => continue,
                // Incoming packet
                Either::First(Ok(Some(inbound))) => {
                    router::dispatch(inbound.topic(), inbound.payload()).await;
                }
                // New packet queued
                Either::Second(packet) => {
                    let packet = packet.0;
                    let mut payload = [0u8; 512];
                    info!("Sending packet: {}", packet);
                    let ser = packet.serialize(&mut payload);
                    match ser  {
                        Ok(len) => {
                            let mut topic: String<64> = String::new();

                            match packet.payload {
                                PacketPayload::Telemetry(_) => { 
                                    mqtt_convention::node_telemetry(&mut topic, client_id) 
                                },
                                PacketPayload::NodeBoot(_) => {
                                    mqtt_convention::node_boot(&mut topic, client_id)
                                },
                                _ => unreachable!(),
                            };

                            let publication = Publication::new(&topic, &payload[..len])
                                .qos(QoS::AtMostOnce);

                            if conn.publish(publication).await.is_err() {
                                error!("Publication error!");
                                break;
                            }

                            info!("Packet sent");
                        },
                        Err(e) => {
                            error!("Serialization error: {}", e);
                        }
                    };
                }
            }
        }
    }
}
