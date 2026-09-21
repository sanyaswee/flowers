//! The dynamic node settings

use serde::{Deserialize, Serialize};

use crate::telemetry::MAX_PLANT_CHANNELS;

/// The main settings struct
#[derive(Copy, Clone, defmt::Format, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[non_exhaustive]
pub struct NodeSettings {
    /// Frequency of creating telemetry packets in seconds
    pub telemetry_packet_creation_freq_s: u16,

    /// Array of settings per each plant channel
    pub plant_settings: [PlantSettings; MAX_PLANT_CHANNELS],

    /// Measurement frequencies
    pub m_freq: MeasurementFrequencies,
}

impl Default for NodeSettings {
    fn default() -> Self {
        Self {
            telemetry_packet_creation_freq_s: 10,
            plant_settings: [PlantSettings::default(); MAX_PLANT_CHANNELS],
            m_freq: MeasurementFrequencies::default(),
        }
    }
}

/// Helper struct that contains all possible measurement frequencies
/// Convention: <quantity>_<unit>
#[derive(Copy, Clone, defmt::Format, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[non_exhaustive]
pub struct MeasurementFrequencies {
    /// Light intensity measurement
    pub light_intensity_s: u16,

    /// BMP / BME measurement frequencies (temp, pressure, humidity)
    pub bmpe_s: u16,
}

impl Default for MeasurementFrequencies {
    fn default() -> Self {
        Self {
            light_intensity_s: 10,
            bmpe_s: 10,
        }
    }
}

/// Settings per each channel
#[derive(Copy, Clone, defmt::Format, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[non_exhaustive]
pub struct PlantSettings {
    /// Channel is enabled
    pub enabled: bool,

    /// Soil moisture measurement frequency in seconds
    /// All time values in embassy are u64, but here we shrink it to u16 for packet size optimization
    pub moisture_m_freq_s: u16,

    /// Watering time (s)
    pub watering_time_s: u16,
}

impl Default for PlantSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            moisture_m_freq_s: 60 * 10, // 10 minutes
            watering_time_s: 5,
        }
    }
}