#![cfg_attr(not(feature = "utoipa"), no_std)]

pub mod node_config;
pub mod telemetry;
pub mod packets;
pub mod node_settings;
pub mod mqtt_convention;