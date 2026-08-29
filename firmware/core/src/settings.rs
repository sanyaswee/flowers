//! This file contains default settings for the node

/// Measurement frequencies
/// Notation - <sensor>_M_FREQ_<unit>
pub const LIGHT_INTENSITY_M_FREQ_S: u64 = 5;
pub const BMPE_M_FREQ_S: u64 = 5; // BMP and BME sensors (temp, pressure, humidity)

/// Cooldowns (<name>_COOLDOWN_<unit>)
pub const TELEMETRY_SENDER_COOLDOWN_MS: u64 = 200;
pub const TELEMETRY_PACKET_CREATION_COOLDOWN_S: u64 = 5;

/// Max size of telemetry channel
/// The channel takes `Packet` structs, whose size is 64 bytes
/// This in fact determines the maximum amount of server downtime that the board could survive
/// The value of 1024 gives us 1.4 hours (assuming packet is created every 5s)
pub const TELEMETRY_CHANNEL_SIZE: usize = 1024;
