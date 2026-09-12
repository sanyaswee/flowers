//! This module contains all packet types

use serde::{Deserialize, Serialize};

use crate::node_config::NodeConfig;
use crate::node_settings::NodeSettings;
use crate::telemetry::NodeTelemetry;

#[derive(Debug, defmt::Format)]
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
    pub const fn new(uptime_ms: u64, payload: PacketPayload) -> Self {
        let header = PacketHeader::new(uptime_ms);
        Self { header, payload }
    }

    /// JSON serialization in order to send over Wi-Fi
    pub fn serialize(&self, buffer: &mut [u8]) -> Result<usize, Error> {
        serde_json_core::to_slice(self, buffer).map_err(Error::from)
    }

    /// JSON deserialization after receiving from Wi-Fi
    pub fn deserialize(data: &[u8]) -> Result<Self, Error> {
        let (packet, _bytes_read) = serde_json_core::from_slice(data)?;
        Ok(packet)
    }
}

/// Packet metadata: node id and timestamp
#[derive(defmt::Format, Serialize, Deserialize, Debug)]
pub struct PacketHeader {
    pub uptime_ms: u64, // real timestamp should be computed on the server side
}

impl PacketHeader {
    const fn new(uptime_ms: u64) -> Self {
        Self { uptime_ms }
    }
}

/// Enum for all packet types
#[derive(defmt::Format, Serialize, Deserialize, Debug)]
#[non_exhaustive]
pub enum PacketPayload {
    /// Standard telemetry packet, sent repeatedly over certain interval
    Telemetry(NodeTelemetry),

    /// The packet sent by the board when it boots
    /// Contains node config. Followed by the settings override packet
    NodeBoot(NodeConfig),

    /// Sent by server after its booting
    /// Followed by a node config packet
    ServerBoot,

    /// Packet sent by server that contains the actual node settings
    SettingsOverride(NodeSettings),
}