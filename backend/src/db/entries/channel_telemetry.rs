//! Module for interaction with `channel_telemetry` DB table

use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use serde::Serialize;
use sqlx::SqlitePool;
use utoipa::ToSchema;

use super::node::NodeEntry;

/// `channel_telemetry` table
#[derive(Serialize, ToSchema)]
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
    pub async fn from_index(
        pool: &SqlitePool,
        node_id: &str,
        channel_id: u8,
        start: Option<NaiveDateTime>,
        end: Option<NaiveDateTime>,
        limit: Option<i64>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let channel_id = channel_id as i64;
        let start_ts =
            start.unwrap_or_else(|| DateTime::from_timestamp_millis(0).unwrap().naive_utc());
        let end_ts = end.unwrap_or_else(|| Utc::now().naive_utc());
        let limit = limit.unwrap_or(100);

        sqlx::query_as!(
            ChannelTelemetryEntry,
            r#"
            SELECT
                id, node_id, channel_id as "channel_id: u8",
                timestamp as "timestamp: NaiveDateTime", uptime as "uptime: i64",
                soil_moisture as "soil_moisture!: f32"
            FROM channel_telemetry
            WHERE node_id = ? AND channel_id = ? AND timestamp >= ? AND timestamp <= ?
            ORDER BY timestamp DESC
            LIMIT ?
            "#,
            node_id,
            channel_id,
            start_ts,
            end_ts,
            limit,
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
