//! MQTT communication topics
//!
//! We use `impl Write` instead of `String` here because this crate is used by `no_std` firmware

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

/// Published by server immediately after it's boot
/// Node responses with its config (NodeBoot)
pub fn server_boot(buf: &mut impl Write) {
    write!(buf, "server/boot").unwrap();
}

/// Override node settings
/// Published by server
pub fn settings_override(buf: &mut impl Write, node_id: &str) {
    write!(buf, "override/{}", node_id).unwrap();
}

/// Water the plant on the channel
/// Published by server
pub fn water(buf: &mut impl Write, node_id: &str) {
    write!(buf, "water/{}", node_id).unwrap();
}
