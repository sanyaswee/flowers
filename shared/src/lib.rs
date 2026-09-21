#![cfg_attr(not(feature = "utoipa"), no_std)]

pub mod mqtt_convention;
pub mod node_config;
pub mod node_settings;
pub mod packets;
pub mod telemetry;

/// Maximum number of plants allowed per node
pub const MAX_PLANT_CHANNELS: usize = 8;
