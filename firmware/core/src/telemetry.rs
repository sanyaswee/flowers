//! Gathering all telemetry and sending it to the server

use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_time::Timer;

use shared::telemetry::NodeTelemetry;

use defmt::info;

use shared::packets::PacketPayload;

use crate::network::channel::{create_packet, PACKET_CHANNEL};
use crate::network::priority::PriorityPacketWrapper;
use crate::settings::DYNAMIC_SETTINGS;

/// Shared telemetry mutex
pub static TELEMETRY: Mutex<ThreadModeRawMutex, NodeTelemetry> = Mutex::new(NodeTelemetry::new());

/// Main broker task
#[embassy_executor::task]
pub async fn gather() {
    let mut settings = DYNAMIC_SETTINGS.receiver().unwrap();
    loop {
        // Needed to drop the lock
        let t = {
            let t = TELEMETRY.lock().await;
            t.clone()
        };
        info!("Telemetry gathered: {}", t);
        let packet = create_packet(PacketPayload::Telemetry(t)).await;
        PACKET_CHANNEL.send(PriorityPacketWrapper(packet)).await;

        let wait = settings.get().await.telemetry_packet_creation_freq_s;
        Timer::after_secs(wait as u64).await;
    }
}