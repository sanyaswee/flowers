//! This module is responsible for api endpoints

use axum::{Json, Router};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;

use sqlx::SqlitePool;

use crate::db::entries::{ChannelEntry, ChannelTelemetryEntry, NodeEntry, NodeTelemetryEntry};

pub fn app(pool: SqlitePool) -> Router {
    Router::new()
        .route("/api/nodes", get(get_all_nodes))
        .route("/api/nodes/{node_id}", get(get_node_by_id))
        .route("/api/channels", get(get_all_channels))
        .route("/api/nodes/{node_id}/channels/{channel_id}", get(get_channel_by_id))
        .route("/api/nodes/{node_id}/telemetry", get(get_node_telemetry))
        .route("/api/nodes/{node_id}/channels/{channel_id}/telemetry", get(get_channel_telemetry))
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