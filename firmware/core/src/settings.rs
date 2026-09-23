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
/// Fun story:
/// When I was adding pump functionality I didn't have the hardware with me,
/// so I was just checking if it builds. Somehow I managed to merge this into master without testing on the physical board.
/// The thing is, the number of receivers on this watch was 5 and the actual number was 7.
/// The result is obvious - HardFault in runtime. Even more funny part - this watch was my first suspect,
/// But since several days have passed I assumed I added 1 extra receiver, not 2.
/// I bumped the watch to 6, but the HardFault did not disappear. So I thought the issue was the packet channel,
/// since `Packet` size in bytes more then doubled after I added new settings and watering packets.
/// Result -> midnight debugging, headache, and nothing fixed the issue.
/// I gave up, opened Claude, gave it the link to the repo, put the effort to high...
/// ...just for it to tell me that I can't count
/// Morality: don't be like me, bump your watches immediately after you add receivers.
pub static DYNAMIC_SETTINGS: Watch<CriticalSectionRawMutex, NodeSettings, 7> = Watch::new();

/// Signal used for providing the new settings
pub static OVERRIDE_SIG: Signal<CriticalSectionRawMutex, NodeSettings> = Signal::new();

/// Task that is responsible for updating settings
#[embassy_executor::task]
pub async fn settings_monitor() {
    let tx = DYNAMIC_SETTINGS.sender();
    // Initialize the default settings
    tx.send(NodeSettings::default());

    loop {
        let new = OVERRIDE_SIG.wait().await;
        tx.send(new);
    }
}
