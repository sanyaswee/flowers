//! Module for interaction with `node_telemetry` DB table

use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use serde::Serialize;
use sqlx::SqlitePool;
use utoipa::ToSchema;

use shared::telemetry::NodeTelemetry;

use super::channel_telemetry::ChannelTelemetryEntry;
use super::node::NodeEntry;

/// `node_telemetry` table
#[derive(Serialize, ToSchema)]
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
    pub async fn from_node_id(
        pool: &SqlitePool,
        node_id: &str,
        start: Option<NaiveDateTime>,
        end: Option<NaiveDateTime>,
        limit: Option<i64>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let start_ts =
            start.unwrap_or_else(|| DateTime::from_timestamp_millis(0).unwrap().naive_utc());
        let end_ts = end.unwrap_or_else(|| Utc::now().naive_utc());
        let limit = limit.unwrap_or(100);
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
            WHERE node_id = ? AND timestamp >= ? AND timestamp <= ?
            ORDER BY timestamp DESC
            LIMIT ?
            "#,
            node_id,
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
        uptime: i64,
        telemetry: NodeTelemetry,
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
                    ChannelTelemetryEntry::push(pool, node_id, i as u8, stamp, plant.soil_moisture)
                        .await?;
                }
            }
        }

        Ok(this)
    }
}
