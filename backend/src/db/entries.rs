//! Structs corresponding to DB tables

use std::time::Duration;

use chrono::{NaiveDateTime, Utc};

use sqlx::SqlitePool;
use sqlx::types::chrono;

use shared::node_config::{NodeConfig, WaterTankDetection, TelemetryCapabilities, NodeId};
use shared::node_settings::{PlantSettings, NodeSettings};

/// `nodes` table
pub struct NodeEntry {
    pub id: i64,
    pub node_id: String,
    pub verbose_name: Option<String>,
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
    pub last_boot: NaiveDateTime,
    pub last_active: NaiveDateTime,
}

impl NodeEntry {
    /// Get DB entry based on node id from packet
    pub async fn from_node_id(pool: &SqlitePool, node_id: NodeId) -> Result<Option<Self>, sqlx::Error> {
        let node_id = node_id.to_string();
        sqlx::query_as!(
            NodeEntry,
            r#"
            SELECT
                id, node_id,
                verbose_name as "verbose_name!: String",
                n_channels as "n_channels: u8",
                water_tank as "water_tank: bool",
                water_tank_level as "water_tank_level: bool",
                temperature as "temperature: bool",
                humidity as "humidity: bool",
                pressure as "pressure: bool",
                light as "light: bool",
                telemetry_report_freq as "telemetry_report_freq: u16",
                light_m_freq as "light_m_freq: u16",
                bmpe_m_freq as "bmpe_m_freq: u16",
                last_boot as "last_boot: NaiveDateTime",
                last_active as "last_active: NaiveDateTime"
            FROM nodes
            WHERE node_id = ?
            "#,
            node_id
        )
            .fetch_optional(pool)
            .await
    }

    /// Push new entry into database
    pub async fn push(
        pool: &SqlitePool,
        node_id: &str,
        uptime: u64,
        config: NodeConfig,
        settings: NodeSettings,
    ) -> Result<Self, sqlx::Error> {
        let water_tank = !matches!(config.water_tank_detection, WaterTankDetection::None);
        let water_tank_level = matches!(config.water_tank_detection, WaterTankDetection::LevelDetection);

        let now = Utc::now().naive_utc();
        let last_boot = now - Duration::from_millis(uptime);
        let last_active = now;

        let n_channels = config.n_plant_channels as i64;
        let telemetry_report_freq = settings.telemetry_packet_creation_freq_s as i64;
        let light_m_freq = settings.m_freq.light_intensity_s as i64;
        let bmpe_m_freq = settings.m_freq.bmpe_s as i64;

        sqlx::query_as!(
            NodeEntry,
            r#"
            INSERT INTO nodes
                (
                   node_id,
                   n_channels,
                   water_tank,
                   water_tank_level,
                   temperature,
                   humidity,
                   pressure,
                   light,
                   telemetry_report_freq,
                   light_m_freq,
                   bmpe_m_freq,
                   last_boot,
                   last_active
                )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING
                id as "id!: i64",
                node_id as "node_id!: String",
                verbose_name as "verbose_name: String",
                n_channels as "n_channels!: u8",
                water_tank as "water_tank!: bool",
                water_tank_level as "water_tank_level!: bool",
                temperature as "temperature!: bool",
                humidity as "humidity!: bool",
                pressure as "pressure!: bool",
                light as "light!: bool",
                telemetry_report_freq as "telemetry_report_freq!: u16",
                light_m_freq as "light_m_freq!: u16",
                bmpe_m_freq as "bmpe_m_freq!: u16",
                last_boot as "last_boot: NaiveDateTime",
                last_active as "last_active: NaiveDateTime"
            "#,
            node_id,
            n_channels,
            water_tank,
            water_tank_level,
            config.telemetry.temperature,
            config.telemetry.humidity,
            config.telemetry.pressure,
            config.telemetry.light,
            telemetry_report_freq,
            light_m_freq,
            bmpe_m_freq,
            last_boot,
            last_active,
        )
            .fetch_one(pool)
            .await
    }

    pub async fn push_default(pool: &SqlitePool, node_id: &str, uptime: u64, config: NodeConfig) -> Result<Self, sqlx::Error> {
        Self::push(pool, node_id, uptime, config, NodeSettings::default()).await
    }

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

    /// Get NodeSettings from entry
    pub async fn get_settings(&self, pool: &SqlitePool) -> Result<NodeSettings, sqlx::Error> {
        let mut settings = NodeSettings::default();

        settings.telemetry_packet_creation_freq_s = self.telemetry_report_freq;
        settings.m_freq.bmpe_s = self.bmpe_m_freq;
        settings.m_freq.light_intensity_s = self.light_m_freq;

        for i in 0..self.n_channels {
            let node_id = self.node_id.as_str();
            match ChannelEntry::from_node(pool, node_id, i).await? {
                Some(ch) => {
                    settings.plant_settings[i as usize] = ch.get_settings();
                },
                None => {
                    ChannelEntry::push_default(pool, node_id, i).await?;
                }
            }
        }

        Ok(settings)
    }
}

/// `channels` table
pub struct ChannelEntry {
    pub id: i64,
    pub node_id: String,
    pub channel_id: u8,
    pub enabled: bool,
    pub moisture_m_freq: u16,
}

impl ChannelEntry {
    /// Get channel by node id and index
    pub async fn from_node(pool: &SqlitePool, node_id: &str, idx: u8) -> Result<Option<Self>, sqlx::Error> {
        let node_id = node_id.to_string();
        let idx = idx as i64;

        sqlx::query_as!(
            ChannelEntry,
            r#"
            SELECT
                id,
                node_id,
                channel_id as "channel_id: u8",
                enabled as "enabled: bool",
                moisture_m_freq as "moisture_m_freq: u16"
            FROM channels
            WHERE node_id = ? AND channel_id = ?
            "#,
            node_id,
            idx
        )
            .fetch_optional(pool)
            .await
    }

    /// Push new entry into the database
    pub async fn push(pool: &SqlitePool, node_id: &str, idx: u8, settings: PlantSettings) -> Result<Self, sqlx::Error> {
        let channel_id = idx as i64;
        let moisture_m_freq = settings.moisture_m_freq_s as i64;

        sqlx::query_as!(
            ChannelEntry,
            r#"
            INSERT INTO channels (node_id, channel_id, enabled, moisture_m_freq)
            VALUES (?, ?, ?, ?)
            RETURNING
                id as "id!: i64",
                node_id as "node_id!: String",
                channel_id as "channel_id!: u8",
                enabled as "enabled!: bool",
                moisture_m_freq as "moisture_m_freq!: u16"
            "#,
            node_id,
            channel_id,
            settings.enabled,
            moisture_m_freq,
        )
            .fetch_one(pool)
            .await
    }

    /// Shortcut for default settings
    pub async fn push_default(pool: &SqlitePool, node_id: &str, idx: u8) -> Result<Self, sqlx::Error> {
        Self::push(pool, node_id, idx, PlantSettings::default()).await
    }



    /// Get channel settings
    pub fn get_settings(&self) -> PlantSettings {
        let mut s = PlantSettings::default();
        s.enabled = self.enabled;
        s.moisture_m_freq_s = self.moisture_m_freq;
        s
    }
}

/// `node_telemetry` table
pub struct NodeTelemetryEntry {
    pub id: u64,
    pub node_id: u64,
    pub timestamp: NaiveDateTime,
    pub uptime: u64,
    pub water_tank_has_water: Option<bool>,
    pub water_tank_level: Option<f32>,
    pub temperature: Option<f32>,
    pub pressure: Option<f32>,
    pub humidity: Option<f32>,
    pub light_intensity: Option<f32>,
}

/// `channel_telemetry` table
pub struct ChannelTelemetryEntry {
    pub id: u64,
    pub node_id: String,
    pub channel_id: u8,
    pub timestamp: NaiveDateTime,
    pub uptime: u64,
    pub soil_moisture: f32,
}