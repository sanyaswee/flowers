//! This module contains NodeConfig and related structs / enums

use serde::{Deserialize, Serialize};

pub type NodeId = heapless::String<32>;

/// Each node should have a configuration defined.
/// Design assumes that each node supports soil moisture reading and pump control on every flower channel
#[derive(defmt::Format, Debug, Deserialize, Serialize)]
pub struct NodeConfig {
    pub node_id: NodeId,
    // Maximum number on flowers attached to this node
    pub n_plant_channels: u8,

    pub water_tank_detection: WaterTankDetection,

    // Node telemetry
    pub telemetry: TelemetryCapabilities,
}

impl NodeConfig {
    pub const fn new(node_id: NodeId, max_flower_channels: u8, water_tank_detection: WaterTankDetection, telemetry: TelemetryCapabilities) -> Self {
        Self {
            node_id,
            n_plant_channels: max_flower_channels, water_tank_detection, telemetry
        }
    }
}

/// Supported telemetry options
#[derive(defmt::Format, Debug, Deserialize, Serialize)]
pub struct TelemetryCapabilities {
    pub temperature: bool,
    pub humidity: bool,
    pub pressure: bool,
    pub light: bool,
}

impl TelemetryCapabilities {
    pub const fn new(temperature: bool, humidity: bool, pressure: bool, light: bool) -> Self {
        Self {
            temperature, humidity, pressure, light
        }
    }
}

/// Supported water tank telemetry capabilities
#[derive(defmt::Format, Debug, Deserialize, Serialize)]
pub enum WaterTankDetection {
    None, // no water tank telemetry
    EmptyDetection, // can detect whether the tank is empty or not
    LevelDetection // can detect the exact water level
}