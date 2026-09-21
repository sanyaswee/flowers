//! Module for interaction with `nodes` DB table

use chrono::{Duration, NaiveDateTime, Utc};
use serde::Serialize;
use sqlx::SqlitePool;
use utoipa::ToSchema;

use shared::node_config::{NodeConfig, TelemetryCapabilities, WaterTankDetection};
use shared::node_settings::NodeSettings;

use super::channel::ChannelEntry;

/// `nodes` table
#[derive(Serialize, ToSchema)]
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
    /// Get all DB entries
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            NodeEntry,
            r#"
            SELECT
                id, node_id, verbose_name as "verbose_name: String",
                n_channels as "n_channels: u8", water_tank as "water_tank: bool",
                water_tank_level as "water_tank_level: bool", temperature as "temperature: bool",
                humidity as "humidity: bool", pressure as "pressure: bool", light as "light: bool",
                telemetry_report_freq as "telemetry_report_freq: u16", light_m_freq as "light_m_freq: u16",
                bmpe_m_freq as "bmpe_m_freq: u16", last_boot as "last_boot: NaiveDateTime",
                last_active as "last_active: NaiveDateTime"
            FROM nodes
            "#
        )
            .fetch_all(pool)
            .await
    }

    /// Get DB entry based on node id from packet
    pub async fn from_node_id(
        pool: &SqlitePool,
        node_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
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
        let water_tank_level = matches!(
            config.water_tank_detection,
            WaterTankDetection::LevelDetection
        );

        let now = Utc::now().naive_utc();
        let last_boot = now - Duration::milliseconds(uptime as i64);
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

    pub async fn push_default(
        pool: &SqlitePool,
        node_id: &str,
        uptime: u64,
        config: NodeConfig,
    ) -> Result<Self, sqlx::Error> {
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
        let telemetry =
            TelemetryCapabilities::new(self.temperature, self.humidity, self.pressure, self.light);
        NodeConfig::new(
            self.node_id.parse().unwrap(),
            self.n_channels,
            tank,
            telemetry,
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
                }
                None => {
                    ChannelEntry::push_default(pool, node_id, i).await?;
                }
            }
        }

        Ok(settings)
    }

    /// Update config in the database
    pub async fn update_config(
        &mut self,
        pool: &SqlitePool,
        config: NodeConfig,
    ) -> Result<(), sqlx::Error> {
        let water_tank = !matches!(config.water_tank_detection, WaterTankDetection::None);
        let water_tank_level = matches!(
            config.water_tank_detection,
            WaterTankDetection::LevelDetection
        );
        let n_channels = config.n_plant_channels as i64;

        sqlx::query!(
            r#"
            UPDATE nodes SET
                n_channels = ?,
                water_tank = ?,
                water_tank_level = ?,
                temperature = ?,
                humidity = ?,
                pressure = ?,
                light = ?
            WHERE node_id = ?
            "#,
            n_channels,
            water_tank,
            water_tank_level,
            config.telemetry.temperature,
            config.telemetry.humidity,
            config.telemetry.pressure,
            config.telemetry.light,
            self.node_id,
        )
        .execute(pool)
        .await?;

        // Update self only if DB write was successful
        self.n_channels = config.n_plant_channels;
        self.water_tank = water_tank;
        self.water_tank_level = water_tank_level;
        self.temperature = config.telemetry.temperature;
        self.humidity = config.telemetry.humidity;
        self.pressure = config.telemetry.pressure;
        self.light = config.telemetry.light;

        Ok(())
    }

    /// Update last boot
    pub async fn update_boot(&mut self, pool: &SqlitePool, uptime: u64) -> Result<(), sqlx::Error> {
        let last_boot = Utc::now().naive_utc() - Duration::milliseconds(uptime as i64);
        sqlx::query!(
            r#"
            UPDATE nodes SET last_boot = ? WHERE node_id = ?
            "#,
            last_boot,
            self.node_id
        )
        .execute(pool)
        .await?;

        self.last_boot = last_boot;

        Ok(())
    }

    /// Update last activity field
    pub async fn update_activity(&mut self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        let now = Utc::now().naive_utc();
        sqlx::query!(
            r#"
            UPDATE nodes SET last_active = ? WHERE node_id = ?
            "#,
            now,
            self.node_id
        )
        .execute(pool)
        .await?;

        // Update locally only if DB operation succeeds
        self.last_active = now;

        Ok(())
    }

    /// Set node verbose name
    pub async fn set_verbose(
        &mut self,
        pool: &SqlitePool,
        name: String,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE nodes SET verbose_name = ? WHERE node_id = ?
            "#,
            name,
            self.node_id
        )
        .execute(pool)
        .await?;

        self.verbose_name = Some(name);

        Ok(())
    }

    /// Update node settings
    pub async fn set_settings(
        &mut self,
        pool: &SqlitePool,
        settings: NodeSettings,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE nodes SET telemetry_report_freq = ?, light_m_freq = ?, bmpe_m_freq = ? WHERE node_id = ?
            "#,
            settings.telemetry_packet_creation_freq_s,
            settings.m_freq.light_intensity_s,
            settings.m_freq.bmpe_s,
            self.node_id,
        )
            .execute(pool)
            .await?;

        // Update node settings
        self.telemetry_report_freq = settings.telemetry_packet_creation_freq_s;
        self.light_m_freq = settings.m_freq.light_intensity_s;
        self.bmpe_m_freq = settings.m_freq.bmpe_s;

        // Update its channel settings
        for i in 0..self.n_channels {
            let channel = ChannelEntry::from_node(pool, &self.node_id, i).await?;
            if let Some(mut channel) = channel {
                channel
                    .set_settings(pool, settings.plant_settings[i as usize])
                    .await?;
            }
        }

        Ok(())
    }
}
