#![no_std]

use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_sync::once_lock::OnceLock;
use shared::node_config::NodeConfig;

pub mod bh1750;
pub mod bmp280;
pub mod telemetry;
pub mod network;
pub mod settings;
pub mod water_tank;
pub mod adc;
pub mod plant_channel;

/// Node configuration declaration
/// This should set this upon boot
pub static NODE_CONFIG: OnceLock<NodeConfig> = OnceLock::new();

/// Mutex for the shared i2c bus
pub type SharedI2C<I2C> = Mutex<NoopRawMutex, I2C>;