//! This module contains `PACKET_CHANNEL` and related helper functions

use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::priority_channel::{PriorityChannel, Max};
use embassy_time::Instant;

use shared::packets::{Packet, PacketPayload};

use crate::network::priority::PriorityPacketWrapper;
use crate::{settings, NODE_CONFIG};

/// The channel for receiving the packets from all over the program
pub(crate) static PACKET_CHANNEL: PriorityChannel<ThreadModeRawMutex, PriorityPacketWrapper, Max, { settings::TELEMETRY_CHANNEL_SIZE  }> = PriorityChannel::new();

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