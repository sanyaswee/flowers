//! Module for interaction with `channels` DB table

use serde::Serialize;
use sqlx::SqlitePool;
use utoipa::ToSchema;

use shared::node_settings::PlantSettings;

/// `channels` table
#[derive(Serialize, ToSchema)]
pub struct ChannelEntry {
    pub id: i64,
    pub node_id: String,
    pub channel_id: u8,
    pub verbose_name: Option<String>,
    pub enabled: bool,
    pub moisture_m_freq: u16,
    pub watering_time: u16,
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
                moisture_m_freq as "moisture_m_freq: u16",
                watering_time as "watering_time: u16"
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
                moisture_m_freq as "moisture_m_freq: u16",
                watering_time as "watering_time: u16"
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
        let watering_time = settings.watering_time_s as i64;

        sqlx::query_as!(
            ChannelEntry,
            r#"
            INSERT INTO channels (node_id, channel_id, enabled, moisture_m_freq, watering_time)
            VALUES (?, ?, ?, ?, ?)
            RETURNING
                id as "id!: i64",
                node_id as "node_id!: String",
                channel_id as "channel_id!: u8",
                verbose_name as "verbose_name!: String",
                enabled as "enabled!: bool",
                moisture_m_freq as "moisture_m_freq!: u16",
                watering_time as "watering_time!: u16"
            "#,
            node_id,
            channel_id,
            settings.enabled,
            moisture_m_freq,
            watering_time,
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