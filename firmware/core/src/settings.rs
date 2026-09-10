//! This file handles node settings

/// ---
/// Non-dynamic (hardcoded) settings
/// ---

/// Minimal cooldown between telemetry packets
pub const TELEMETRY_SENDER_COOLDOWN_MS: u64 = 200;

/// Max size of telemetry channel
/// This, along with the packet size and packet creation frequency determines the time that the node can survive without server
pub const TELEMETRY_CHANNEL_SIZE: usize = 1024;

/// ---
/// Dynamic (server-adjustable) settings
/// ---

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_sync::watch::Watch;

use shared::node_settings::NodeSettings;

/// Watch that contains current settings
pub static DYNAMIC_SETTINGS: Watch<CriticalSectionRawMutex, NodeSettings, 5> = Watch::new();

/// Signal used for providing the new settings
pub static OVERWRITE_SIG: Signal<CriticalSectionRawMutex, NodeSettings> = Signal::new();

/// Task that is responsible for updating settings
#[embassy_executor::task]
pub async fn settings_monitor() {
    let tx = DYNAMIC_SETTINGS.sender();
    // Initialize the default settings
    tx.send(NodeSettings::default());
    
    loop {
        let new = OVERWRITE_SIG.wait().await;
        tx.send(new);
    }
}
