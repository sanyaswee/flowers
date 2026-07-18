//! This module contains telemetry related structs

use defmt::Format;

/// Node specific telemetry
#[derive(Format, Clone)]
pub struct NodeTelemetry {
    /// Water tank telemetry
    pub water_tank_has_water: Option<bool>,
    pub water_tank_level: Option<f32>,

    /// Other telemetry
    pub temperature: Option<f32>,
    pub pressure: Option<f32>,
    pub air_humidity: Option<f32>,
    pub light_intensity: Option<f32>
}

impl NodeTelemetry {
    pub const fn new() -> NodeTelemetry {
        NodeTelemetry {
            water_tank_has_water: None,
            water_tank_level: None,
            temperature: None,
            pressure: None,
            air_humidity: None,
            light_intensity: None,
        }
    }
}