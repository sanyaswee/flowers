//! This module is responsible for tracking Wi-Fi and server connection

use embedded_hal::digital::OutputPin;

use embassy_futures::select::{select, Either};

use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, RawMutex};
use embassy_sync::mutex::Mutex;
use embassy_sync::watch::Watch;

use embassy_time::{Duration, Timer};

use crate::wifi_broker::PacketSender;

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
pub async fn track<P: OutputPin>(mut led: P) {
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