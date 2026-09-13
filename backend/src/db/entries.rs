//! Structs corresponding to DB tables

use shared::node_config::{NodeConfig, WaterTankDetection, TelemetryCapabilities};
use shared::node_settings::{PlantSettings, NodeSettings};

/// `nodes` table
pub struct NodeEntry {
    pub id: u64,
    pub node_id: String,
    pub verbose_name: String,
    pub n_channels: u8,
    pub water_tank: bool,
    pub water_tank_level: bool,
    pub temperature: bool,
    pub humidity: bool,
    pub pressure: bool,
    pub light: bool,
    pub telemetry_report_freq: u16,
    pub light_m_freq: u16,
    pub bmpe_m_freq: u16,
    // TODO
    // pub last_boot: ,
    // pub last_active: ,
}

impl NodeEntry {
    /// Convert into NodeConfig
    pub fn get_config(&self) -> NodeConfig {
        let tank = if self.water_tank {
            if self.water_tank_level {
                WaterTankDetection::LevelDetection
            } else {
                WaterTankDetection::EmptyDetection
            }
        } else {
            WaterTankDetection::None
        };
        let telemetry = TelemetryCapabilities::new(
            self.temperature,
            self.humidity,
            self.pressure,
            self.light
        );
        NodeConfig::new(
            self.node_id.parse().unwrap(),
            self.n_channels,
            tank,
            telemetry
        )
    }

    pub fn get_settings(&self) -> NodeSettings {
        todo!()
    }
}

impl Into<NodeConfig> for NodeEntry {
    fn into(self) -> NodeConfig {
        self.get_config()
    }
}

/// `channels` table
pub struct ChannelEntry {
    pub id: u64,
    pub node_id: String,
    pub channel_id: u8,
    pub enabled: bool,
    pub moisture_m_freq: u16,
}

impl ChannelEntry {
    pub fn get_settings(&self) -> PlantSettings {
        let mut s = PlantSettings::default();
        s.enabled = self.enabled;
        s.moisture_m_freq_s = self.moisture_m_freq;
        s
    }
}

impl Into<PlantSettings> for ChannelEntry {
    fn into(self) -> PlantSettings {
        self.get_settings()
    }
}