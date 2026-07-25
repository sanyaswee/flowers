#![no_std]

use embassy_sync::once_lock::OnceLock;
use shared::node_config::NodeConfig;

pub mod bh1750;
pub mod i2c_mutex;
pub mod bmp280;
pub mod telemetry_broker;

/// Node configuration declaration
pub static NODE_CONFIG: OnceLock<NodeConfig> = OnceLock::new();