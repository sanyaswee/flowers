//! This module contains telemetry related structs

use serde::{Serialize, Deserialize};

/// Maximum number of plants allowed per node
pub(crate) const MAX_PLANT_CHANNELS: usize = 8;

/// Node specific telemetry
#[derive(defmt::Format, Clone, Serialize, Deserialize, Debug)]
pub struct NodeTelemetry {
    /// Water tank telemetry
    pub water_tank_has_water: Option<bool>,
    pub water_tank_level: Option<f32>,

    /// Other telemetry
    pub temperature: Option<f32>,
    pub pressure: Option<f32>,
    pub air_humidity: Option<f32>,
    pub light_intensity: Option<f32>,

    /// Plant channel telemetry
    pub plant_telemetry: [Option<PlantTelemetry>; MAX_PLANT_CHANNELS],
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
            plant_telemetry: [None; MAX_PLANT_CHANNELS],
        }
    }
}

/// Plant specific telemetry
#[derive(defmt::Format, Clone, Serialize, Deserialize, Debug, Copy)]
pub struct PlantTelemetry {
    /// Timestamp of last measurement, needed to avoid bloating the db with it
    pub measurement_stamp: u64,
    /// Actual measurement
    pub soil_moisture: f32,
}

impl PlantTelemetry {
    pub const fn new(stamp: u64, moisture: f32) -> Self {
        Self { measurement_stamp: stamp, soil_moisture: moisture }
    }
}