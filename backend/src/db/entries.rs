//! Structs corresponding to DB tables

use chrono::{NaiveDateTime, Duration, Utc};
use serde::Serialize;
use sqlx::SqlitePool;

use shared::node_config::{NodeConfig, WaterTankDetection, TelemetryCapabilities};
use shared::node_settings::{PlantSettings, NodeSettings};
use shared::telemetry::NodeTelemetry;

/// `nodes` table
#[derive(Serialize)]
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
    pub async fn from_node_id(pool: &SqlitePool, node_id: &str) -> Result<Option<Self>, sqlx::Error> {
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

    /// Update config in the database
    pub async fn update_config(&mut self, pool: &SqlitePool, config: NodeConfig) -> Result<(), sqlx::Error> {
        let water_tank = !matches!(config.water_tank_detection, WaterTankDetection::None);
        let water_tank_level = matches!(config.water_tank_detection, WaterTankDetection::LevelDetection);
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
    pub async fn set_verbose(&mut self, pool: &SqlitePool, name: String) -> Result<(), sqlx::Error> {
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
    pub async fn set_settings(&mut self, pool: &SqlitePool, settings: NodeSettings) -> Result<(), sqlx::Error> {
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
            if channel.is_some() {
                channel.unwrap().set_settings(pool, settings.plant_settings[i as usize]).await?;
            }
        }

        Ok(())
    }
}

/// `channels` table
#[derive(Serialize)]
pub struct ChannelEntry {
    pub id: i64,
    pub node_id: String,
    pub channel_id: u8,
    pub verbose_name: Option<String>,
    pub enabled: bool,
    pub moisture_m_freq: u16,
}

impl ChannelEntry {
    /// Get all DB entries
    pub async fn get_all(pool: &SqlitePool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            ChannelEntry,
            r#"
            SELECT
                id, node_id, channel_id as "channel_id: u8",
                verbose_name as "verbose_name: String", enabled as "enabled: bool",
                moisture_m_freq as "moisture_m_freq: u16"
            FROM channels
            "#
        )
            .fetch_all(pool)
            .await
    }

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
                verbose_name as "verbose_name!: String",
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
                verbose_name as "verbose_name!: String",
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

    /// Set node verbose name
    pub async fn set_verbose(&mut self, pool: &SqlitePool, name: String) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE channels SET verbose_name = ? WHERE node_id = ? AND channel_id = ?
            "#,
            name,
            self.node_id,
            self.channel_id,
        )
            .execute(pool)
            .await?;

        self.verbose_name = Some(name);

        Ok(())
    }

    /// Enable channel
    pub async fn enable(&mut self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        self.set_enable(pool, true).await
    }

    /// Disable channel
    pub async fn disable(&mut self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        self.set_enable(pool, false).await
    }

    /// Enable / disable
    pub async fn set_enable(&mut self, pool: &SqlitePool, enable: bool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE channels SET enabled = ? WHERE node_id = ? AND channel_id = ?
            "#,
            enable,
            self.node_id,
            self.channel_id,
        )
            .execute(pool)
            .await?;

        self.enabled = enable;

        Ok(())
    }

    /// Set soil moisture measurement frequency
    pub async fn set_moisture_m_freq(&mut self, pool: &SqlitePool, value: u16) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE channels SET moisture_m_freq = ? WHERE node_id = ? AND channel_id = ?
            "#,
            value,
            self.node_id,
            self.channel_id,
        )
            .execute(pool)
            .await?;

        self.moisture_m_freq = value;

        Ok(())
    }

    /// Set settings (PlantSettings)
    pub async fn set_settings(&mut self, pool: &SqlitePool, settings: PlantSettings) -> Result<(), sqlx::Error> {
        self.set_enable(pool, settings.enabled).await?;
        self.set_moisture_m_freq(pool, settings.moisture_m_freq_s).await?;

        Ok(())
    }
}


/// `node_telemetry` table
#[derive(Serialize)]
pub struct NodeTelemetryEntry {
    pub id: i64,
    pub node_id: String,
    pub timestamp: NaiveDateTime,
    pub uptime: i64,
    pub water_tank_has_water: Option<bool>,
    pub water_tank_level: Option<f32>,
    pub temperature: Option<f32>,
    pub pressure: Option<f32>,
    pub humidity: Option<f32>,
    pub light_intensity: Option<f32>,
}

impl NodeTelemetryEntry {
    /// Get all entries from node
    pub async fn from_node_id(pool: &SqlitePool, node_id: &str) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            NodeTelemetryEntry,
            r#"
            SELECT
                id, node_id, timestamp as "timestamp: NaiveDateTime", uptime as "uptime: i64",
                water_tank_has_water as "water_tank_has_water: bool",
                water_tank_level as "water_tank_level: f32",
                temperature as "temperature: f32",
                pressure as "pressure: f32",
                humidity as "humidity: f32",
                light_intensity as "light_intensity: f32"
            FROM node_telemetry
            WHERE node_id = ?
            ORDER BY timestamp DESC
            LIMIT 100
            "#,
            node_id
        )
            .fetch_all(pool)
            .await
    }
    /// Push new entry into the database
    pub async fn push(
        pool: &SqlitePool,
        node_id: &str,
        uptime: i64,
        telemetry: NodeTelemetry
    ) -> Result<Self, sqlx::Error> {
        let node = NodeEntry::from_node_id(pool, node_id).await?;
        assert!(node.is_some());

        let mut node = node.unwrap();
        node.update_activity(pool).await?;

        let timestamp = node.last_boot + Duration::milliseconds(uptime);
        let this = sqlx::query_as!(
            NodeTelemetryEntry,
            r#"
            INSERT INTO node_telemetry
                (node_id, timestamp, uptime, water_tank_has_water, water_tank_level, temperature, pressure, humidity, light_intensity)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING
                id as "id!: i64",
                node_id as "node_id!: String",
                timestamp as "timestamp!: NaiveDateTime",
                uptime as "uptime!: i64",
                water_tank_has_water as "water_tank_has_water!: bool",
                water_tank_level as "water_tank_level!: f32",
                temperature as "temperature!: f32",
                pressure as "pressure!: f32",
                humidity as "humidity!: f32",
                light_intensity as "light_intensity!: f32"
            "#,
            node_id,
            timestamp,
            uptime,
            telemetry.water_tank_has_water,
            telemetry.water_tank_level,
            telemetry.temperature,
            telemetry.pressure,
            telemetry.air_humidity,
            telemetry.light_intensity,
        )
            .fetch_one(pool)
            .await?;

        // Push channel telemetry if needed
        for (i, plant) in telemetry.plant_telemetry.iter().enumerate() {
            if plant.is_some() {
                let plant = plant.unwrap();
                let stamp = plant.measurement_stamp as i64;
                if ChannelTelemetryEntry::is_new(pool, node_id, i as u8, stamp).await? {
                    println!("Updating telemetry");
                    ChannelTelemetryEntry::push(pool, node_id, i as u8, stamp, plant.soil_moisture).await?;
                }
            }
        }

        Ok(this)
    }
}

/// `channel_telemetry` table
#[derive(Serialize)]
pub struct ChannelTelemetryEntry {
    pub id: i64,
    pub node_id: String,
    pub channel_id: u8,
    pub timestamp: NaiveDateTime,
    pub uptime: i64,
    pub soil_moisture: f32,
}

impl ChannelTelemetryEntry {
    /// Get all entries for channel
    pub async fn from_index(pool: &SqlitePool, node_id: &str, channel_id: u8) -> Result<Vec<Self>, sqlx::Error> {
        let channel_id = channel_id as i64;
        sqlx::query_as!(
            ChannelTelemetryEntry,
            r#"
            SELECT
                id, node_id, channel_id as "channel_id: u8",
                timestamp as "timestamp: NaiveDateTime", uptime as "uptime: i64",
                soil_moisture as "soil_moisture!: f32"
            FROM channel_telemetry
            WHERE node_id = ? AND channel_id = ?
            ORDER BY timestamp DESC
            LIMIT 100
            "#,
            node_id,
            channel_id
        )
            .fetch_all(pool)
            .await
    }

    /// Push new entry into the database
    pub async fn push(
        pool: &SqlitePool,
        node_id: &str,
        channel_id: u8,
        uptime_stamp: i64,
        moisture: f32,
    ) -> Result<Self, sqlx::Error> {
        let node = NodeEntry::from_node_id(pool, node_id).await?;
        assert!(node.is_some());

        let node = node.unwrap();
        let timestamp = node.last_boot + Duration::milliseconds(uptime_stamp);

        sqlx::query_as!(
            ChannelTelemetryEntry,
            r#"
            INSERT INTO channel_telemetry (node_id, channel_id, timestamp, uptime, soil_moisture)
            VALUES (?, ?, ?, ?, ?)
            RETURNING
                id as "id!: i64",
                node_id as "node_id!: String",
                channel_id as "channel_id!: u8",
                timestamp as "timestamp!: NaiveDateTime",
                uptime as "uptime!: i64",
                soil_moisture as "soil_moisture!: f32"
            "#,
            node_id,
            channel_id,
            timestamp,
            uptime_stamp,
            moisture,
        )
            .fetch_one(pool)
            .await
    }

    /// Check if telemetry is already written
    pub async fn is_new(
        pool: &SqlitePool,
        node_id: &str,
        channel_id: u8,
        uptime_stamp: i64,
    ) -> Result<bool, sqlx::Error> {
        let node = NodeEntry::from_node_id(pool, node_id).await?;
        assert!(node.is_some());

        let node = node.unwrap();
        let timestamp = node.last_boot + Duration::milliseconds(uptime_stamp);

        let row = sqlx::query!(
            r#"
            SELECT * FROM channel_telemetry
            WHERE node_id = ? AND channel_id = ? AND timestamp = ?
            "#,
            node_id,
            channel_id,
            timestamp,
        )
            .fetch_optional(pool)
            .await?;

        match row {
            Some(_) => Ok(false),
            None => Ok(true),
        }
    }
}