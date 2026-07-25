//! This module contains all packet types

use crate::telemetry::NodeTelemetry;

/// Standard telemetry packet, sent repeatedly over certain interval
#[derive(defmt::Format)]
pub struct TelemetryPacket {
    pub node_id: u64,
    /// Uptime in ms, real timestamp should be computed on the server side
    pub uptime_ms: u64,
    pub node_telemetry: NodeTelemetry,
}

impl TelemetryPacket {
    pub const fn new(node_id: u64, uptime_ms: u64, node_telemetry: NodeTelemetry) -> Self {
        Self { node_id, uptime_ms, node_telemetry }
    }
}