//! This module handles network operations

mod router;
pub mod status;
pub(crate) mod priority;
pub mod provider;
pub(crate) mod channel;

use defmt::{error, info};

use embassy_futures::select::{select, Either};
use embassy_time::{Timer};

use heapless::String;

use minimq::{Buffers, ConfigBuilder, Session, QoS, Publication};

use shared::mqtt_convention;
use shared::packets::{Packet, PacketPayload};

use provider::TcpProvider;
use status::{NetworkStatus, NETWORK_STATUS};
use channel::{PACKET_CHANNEL, push_boot};

/// The MQTT task
pub async fn mqtt_task<T: TcpProvider>(mut tcp: T, client_id: &str) {
    let tx = NETWORK_STATUS.sender();

    // Node has just booted
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
        // Overcomplicated due to Rust lifetimes constraints
        // bufs contains strings, router::topics() writes actual topics to the strings
        let mut bufs = [const { String::new() }; router::TOPICS_NUM];
        let topics = router::topics(&mut bufs).await;
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
