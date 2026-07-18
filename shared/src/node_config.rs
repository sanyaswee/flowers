//! This module contains NodeConfig and related structs / enums

/// Each node should have a configuration defined.
/// Design assumes that each node supports soil moisture reading and pump control on every flower channel
pub struct NodeConfig {
    // Maximum number on flowers attached to this node
    max_flower_channels: u8,

    water_tank_detection: WaterTankDetection,

    // Node telemetry
    telemetry: TelemetryCapabilities,
}

/// Supported telemetry options
pub struct TelemetryCapabilities {
    pub temperature: bool,
    pub humidity: bool,
    pub pressure: bool,
    pub light: bool,
}

/// Supported water tank telemetry capabilities
pub enum WaterTankDetection {
    None, // no water tank telemetry
    EmptyDetection, // can detect whether the tank is empty or not
    LevelDetection // can detect the exact water level
}