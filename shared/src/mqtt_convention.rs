//! MQTT communication topics

use core::fmt::Write;

/// Telemetry published by node, containing node ID
pub fn node_telemetry(buf: &mut impl Write, node_id: &str) {
    write!(buf, "node/{}/telemetry", node_id).unwrap();
}

/// Sent on node boot and after server boot
/// Contains NodeConfig
pub fn node_boot(buf: &mut impl Write, node_id: &str) {
    write!(buf, "node/{}/boot", node_id).unwrap();
}