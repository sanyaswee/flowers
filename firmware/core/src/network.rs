//! This module contains hardware-generic network traits and tasks

use defmt::{error, info};
use embassy_futures::select::{select, Either};

use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, RawMutex, ThreadModeRawMutex};
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use embassy_sync::watch::Watch;

use embassy_time::{Duration, Instant, Timer};

use embedded_hal::digital::OutputPin;

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
pub static TELEMETRY_CHANNEL: Channel<ThreadModeRawMutex, Packet, 16> = Channel::new();

/// This trait should be implemented in the node specific crate
pub trait PacketSender {
    type Error;

    /// Send the packet to the server
    async fn send(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;

    /// (Re)connect to the server
    async fn reconnect(&mut self) -> Result<(), Self::Error>;
}

/// Function to create a packet from generic payload
pub async fn create_packet(payload: PacketPayload) -> Packet {
    let id = NODE_CONFIG.get().await.node_id;
    let uptime = Instant::now().as_millis();

    Packet::new(id, uptime, payload)
}

/// Constantly try reconnecting to the server if HostNotFound 
pub async fn auto_reconnect<M, S>(sender: &Mutex<M, S>)
where
    M: RawMutex,
    S: PacketSender,
{
    let tx = NETWORK_STATUS.sender();
    let mut rx = NETWORK_STATUS.receiver().unwrap();
    loop {
        let state = rx.changed().await;
        if state != NetworkStatus::HostNotFound {
            continue;
        }

        let mut cooldown = Duration::from_secs(1);
        loop {
            let result = sender.lock().await.reconnect().await;
            if result.is_ok() {
                tx.send(NetworkStatus::Connected);
                break;
            }

            match select(Timer::after(cooldown), rx.changed()).await {
                Either::First(_) => {
                    cooldown = (cooldown * 2).min(Duration::from_secs(30));
                }
                Either::Second(new) => {
                    if new != NetworkStatus::HostNotFound {
                        break;
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

/// The task to send the telemetry to the server
/// TODO queue packets if server is down
pub async fn telemetry_sender<M, S>(sender: &Mutex<M, S>)
where
    M: RawMutex,
    S: PacketSender, <S as PacketSender>::Error: defmt::Format
{
    let tx = NETWORK_STATUS.sender();
    let mut rx = NETWORK_STATUS.receiver().unwrap();

    loop {
        let packet = TELEMETRY_CHANNEL.receive().await;
        info!("Telemetry packet processing: {}", packet);

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

        Timer::after_millis(settings::TELEMETRY_SENDER_COOLDOWN_MS).await;
    }
}
