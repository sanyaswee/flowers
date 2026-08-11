//! This module contains all packet types

use serde::{Deserialize, Serialize};

use crate::node_config::NodeId;
use crate::telemetry::NodeTelemetry;

#[derive(Debug)]
pub enum Error {
    Serialize,
    Deserialize,
}

impl From<serde_json_core::ser::Error> for Error {
    fn from(_: serde_json_core::ser::Error) -> Self {
        Error::Serialize
    }
}

impl From<serde_json_core::de::Error> for Error {
    fn from(_: serde_json_core::de::Error) -> Self {
        Error::Deserialize
    }
}

/// The generic-ish packet type
#[derive(defmt::Format, Serialize, Deserialize, Debug)]
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
        serde_json_core::to_slice(self, buffer).map_err(Error::from)
    }

    /// JSON deserialization after receiving from Wifi
    pub fn deserialize(data: &[u8]) -> Result<Self, Error> {
        let (packet, _bytes_read) = serde_json_core::from_slice(data)?;
        Ok(packet)
    }
}

/// Packet metadata: node id and timestamp
#[derive(defmt::Format, Serialize, Deserialize, Debug)]
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
#[derive(defmt::Format, Serialize, Deserialize, Debug)]
pub enum PacketPayload {
    /// Standard telemetry packet, sent repeatedly over certain interval
    Telemetry(NodeTelemetry)
}