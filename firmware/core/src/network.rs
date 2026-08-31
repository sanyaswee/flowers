//! This module contains hardware-generic network traits and tasks

use core::fmt::Write as _;

use defmt::info;

use embassy_futures::select::{select, Either};

use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, RawMutex, ThreadModeRawMutex};
use embassy_sync::channel::Channel;
use embassy_sync::watch::Watch;

use embassy_time::{Instant, Timer};

use embedded_hal::digital::OutputPin;

use embedded_io_async::{Read, Write};

use heapless::String;

use minimq::{Buffers, ConfigBuilder, Session, QoS, Publication};

use shared::packets::{Packet, PacketPayload};

use crate::NODE_CONFIG;
use crate::settings;

/// Enum with all possible network statuses
#[derive(Clone, PartialEq)]
pub enum NetworkStatus {
    /// Completely disconnected from Wi-Fi (default state on boot)
    /// Indicated by the blinking LED
    Disconnected,

    /// Connected to the network, but could not locate the backend host
    /// Indicated by the shining LED
    HostNotFound,

    /// Successfully connected to host
    /// Indicated by turned off LED
    Connected
}

/// Global-accessible network status
pub static NETWORK_STATUS: Watch<CriticalSectionRawMutex, NetworkStatus, 3> = Watch::new();

/// The channel for receiving the telemetry from telemetry_broker
pub static TELEMETRY_CHANNEL: Channel<ThreadModeRawMutex, Packet, { settings::TELEMETRY_CHANNEL_SIZE  }> = Channel::new();

/// Should be implemented in the node specific crate
pub trait TcpProvider {
    type Stream: Read + Write;
    type Error: defmt::Format;

    /// Establish a raw TCP connection to the broker
    async fn connect(&mut self) -> Result<Self::Stream, Self::Error>;
}

/// Function to create a packet from generic payload
pub async fn create_packet(payload: PacketPayload) -> Packet {
    let id = NODE_CONFIG.get().await.node_id;
    let uptime = Instant::now().as_millis();

    Packet::new(id, uptime, payload)
}

/// The MQTT task
pub async fn mqtt_network_task<T: TcpProvider>(mut tcp: T, client_id: &str) {
    let tx = NETWORK_STATUS.sender();

    // Allocate minimq 0.13.0 buffers directly on the task stack
    let mut rx_buf = [0u8; 256];
    let mut tx_buf = [0u8; 768];
    let buffers = Buffers::new(&mut rx_buf, &mut tx_buf);

    let config = ConfigBuilder::new(buffers)
        .client_id(client_id).unwrap()
        .keepalive_interval(60);
    let mut session = Session::new(config);

    loop {
        tx.send(NetworkStatus::HostNotFound);

        // raw TCP stream
        let stream = match tcp.connect().await {
            Ok(s) => s,
            Err(_) => {
                Timer::after_secs(2).await;
                continue;
            }
        };

        // core executes MQTT Handshake
        let mut conn = match session.connect(stream).await {
            Ok(c) => c,
            Err(_) => continue,
        };

        tx.send(NetworkStatus::Connected);

        // core drives polling and publishes telemetry
        loop {
            match select(conn.poll(), TELEMETRY_CHANNEL.receive()).await {
                // Connection or protocol error, drop and reconnect
                Either::First(Err(_)) => break,
                // Idle poll success
                Either::First(Ok(_)) => continue,
                // New packet queued
                Either::Second(packet) => {
                    let mut payload = [0u8; 256];
                    info!("Sending packet: {}", packet);
                    if let Ok(len) = packet.serialize(&mut payload) {

                        let mut topic: String<64> = String::new();
                        write!(&mut topic, "node/{}/telemetry", client_id).unwrap();

                        let publication = Publication::new(&topic, &payload[..len])
                            .qos(QoS::AtMostOnce);

                        if conn.publish(publication).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    }
}

/// Function to track network status
/// Should be called before Wi-Fi initialization
pub async fn track_status<P: OutputPin>(mut led: P) {
    let tx = NETWORK_STATUS.sender();
    let mut rx = NETWORK_STATUS.receiver().unwrap();

    // Init default state
    tx.send(NetworkStatus::Disconnected);
    let mut state = rx.get().await;

    loop {
        match state {
            NetworkStatus::Disconnected => {
                // Blink LED
                let blink_future = async {
                    loop {
                        led.set_low().unwrap();
                        Timer::after_millis(500).await;
                        led.set_high().unwrap();
                        Timer::after_millis(500).await;
                    }
                };
                match select(blink_future, rx.changed()).await {
                    Either::First(_) => unreachable!(),
                    Either::Second(new) => {
                        state = new;
                        continue;
                    }
                }
            },
            NetworkStatus::HostNotFound => {
                // LED on
                led.set_high().unwrap();
            },
            NetworkStatus::Connected => {
                // LED off
                led.set_low().unwrap();
            }
        }

        state = rx.changed().await;
    }
}
