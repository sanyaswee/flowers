//! This module is responsible for tracking Wi-Fi and server connection

use embedded_hal::digital::OutputPin;

use embassy_futures::select::{select, Either};

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::watch::Watch;

use embassy_time::Timer;

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
pub static NETWORK_STATUS: Watch<CriticalSectionRawMutex, NetworkStatus, 2> = Watch::new();

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