//! This module is responsible for api endpoints

use axum::{Json, Router};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};

use serde::Deserialize;
use sqlx::SqlitePool;

use crate::db::entries::{ChannelEntry, ChannelTelemetryEntry, NodeEntry, NodeTelemetryEntry};

#[derive(Deserialize)]
pub struct VerboseNamePayload {
    pub verbose_name: String,
}

pub fn app(pool: SqlitePool) -> Router {
    Router::new()
        // Getters
        .route("/api/nodes", get(get_all_nodes))
        .route("/api/nodes/{node_id}", get(get_node_by_id))
        .route("/api/channels", get(get_all_channels))
        .route("/api/nodes/{node_id}/channels/{channel_id}", get(get_channel_by_id))
        .route("/api/nodes/{node_id}/telemetry", get(get_node_telemetry))
        .route("/api/nodes/{node_id}/channels/{channel_id}/telemetry", get(get_channel_telemetry))
        // Setters
        .route("/api/nodes/{node_id}/verbose", post(set_node_verbose_name))
        .route("/api/nodes/{node_id}/channels/{channel_id}/verbose", post(set_channel_verbose_name))
        .route("/api/nodes/{node_id}/channels/{channel_id}/enable", post(enable_channel))
        .route("/api/nodes/{node_id}/channels/{channel_id}/disable", post(disable_channel))
        .with_state(pool)
}

async fn get_all_nodes(State(pool): State<SqlitePool>) -> Result<Json<Vec<NodeEntry>>, StatusCode> {
    NodeEntry::get_all(&pool)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_node_by_id(
    State(pool): State<SqlitePool>,
    Path(node_id): Path<String>,
) -> Result<Json<NodeEntry>, StatusCode> {
    match NodeEntry::from_node_id(&pool, &node_id).await {
        Ok(Some(node)) => Ok(Json(node)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_all_channels(State(pool): State<SqlitePool>) -> Result<Json<Vec<ChannelEntry>>, StatusCode> {
    ChannelEntry::get_all(&pool)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_channel_by_id(
    State(pool): State<SqlitePool>,
    Path((node_id, channel_id)): Path<(String, u8)>,
) -> Result<Json<ChannelEntry>, StatusCode> {
    match ChannelEntry::from_node(&pool, &node_id, channel_id).await {
        Ok(Some(channel)) => Ok(Json(channel)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_node_telemetry(
    State(pool): State<SqlitePool>,
    Path(node_id): Path<String>,
) -> Result<Json<Vec<NodeTelemetryEntry>>, StatusCode> {
    NodeTelemetryEntry::from_node_id(&pool, &node_id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_channel_telemetry(
    State(pool): State<SqlitePool>,
    Path((node_id, channel_id)): Path<(String, u8)>,
) -> Result<Json<Vec<ChannelTelemetryEntry>>, StatusCode> {
    ChannelTelemetryEntry::from_index(&pool, &node_id, channel_id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn set_node_verbose_name(
    State(pool): State<SqlitePool>,
    Path(node_id): Path<String>,
    Json(payload): Json<VerboseNamePayload>,
) -> Result<StatusCode, StatusCode> {
    let mut node = match NodeEntry::from_node_id(&pool, &node_id).await {
        Ok(Some(node)) => node,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    match node.set_verbose(&pool, payload.verbose_name).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn set_channel_verbose_name(
    State(pool): State<SqlitePool>,
    Path((node_id, channel_id)): Path<(String, u8)>,
    Json(payload): Json<VerboseNamePayload>,
) -> Result<StatusCode, StatusCode> {
    let mut channel = match ChannelEntry::from_node(&pool, &node_id, channel_id).await {
        Ok(Some(channel)) => channel,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    match channel.set_verbose(&pool, payload.verbose_name).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn enable_channel(
    State(pool): State<SqlitePool>,
    Path((node_id, channel_id)): Path<(String, u8)>,
) -> Result<StatusCode, StatusCode> {
    let mut channel = match ChannelEntry::from_node(&pool, &node_id, channel_id).await {
        Ok(Some(channel)) => channel,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    match channel.enable(&pool).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn disable_channel(
    State(pool): State<SqlitePool>,
    Path((node_id, channel_id)): Path<(String, u8)>,
) -> Result<StatusCode, StatusCode> {
    let mut channel = match ChannelEntry::from_node(&pool, &node_id, channel_id).await {
        Ok(Some(channel)) => channel,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    match channel.disable(&pool).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}