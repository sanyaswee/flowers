//! This module contains all packet types

use crate::node_config::NodeId;
use crate::telemetry::NodeTelemetry;

/// The generic-ish packet type
#[derive(defmt::Format)]
pub struct Packet {
    pub header: PacketHeader,
    pub payload: PacketPayload,
}

impl Packet {
    pub const fn new(node_id: NodeId, uptime_ms: u64, payload: PacketPayload) -> Self {
        let header = PacketHeader::new(node_id, uptime_ms);
        Self { header, payload }
    }

    /// JSON serialization in order to send over WiFi
    pub fn serialize(&self, buffer: &mut [u8]) -> Result<usize, Error> {
        // TODO
    }

    /// JSON deserialization after receiving from Wifi
    pub fn deserialize() -> Result<Self, Error> {
        // TODO
    }
}

/// Packet metadata: node id and timestamp
#[derive(defmt::Format)]
pub struct PacketHeader {
    pub node_id: NodeId,
    pub uptime_ms: u64, // real timestamp should be computed on the server side
}

impl PacketHeader {
    const fn new(node_id: NodeId, uptime_ms: u64) -> Self {
        Self { node_id, uptime_ms }
    }
}

/// Enum for all packet types
#[derive(defmt::Format)]
pub enum PacketPayload {
    /// Standard telemetry packet, sent repeatedly over certain interval
    Telemetry(NodeTelemetry)
}