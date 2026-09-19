//! This module is responsible for API endpoints

use std::sync::Arc;

use axum::{Json, Router};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};

use chrono::NaiveDateTime;

use rumqttc::AsyncClient;
use serde::Deserialize;
use sqlx::SqlitePool;
use utoipa::{ToSchema, OpenApi, IntoParams};
use utoipa_swagger_ui::SwaggerUi;

use shared::node_settings::{NodeSettings, PlantSettings, MeasurementFrequencies};

use crate::db::entries::*;
use crate::mqtt::router::override_settings;

#[derive(OpenApi)]
#[openapi(
    paths(
        // Getters
        get_all_nodes,
        get_node_by_id,
        get_all_channels,
        get_channel_by_id,
        get_node_telemetry,
        get_channel_telemetry,

        // Setters
        set_node_verbose_name,
        set_channel_verbose_name,
        enable_channel,
        disable_channel,
        set_node_settings,
        set_channel_settings
    ),
    components(
        schemas(
            // Database models
            NodeEntry,
            ChannelEntry,
            NodeTelemetryEntry,
            ChannelTelemetryEntry,

            // API payloads and filters
            VerboseNamePayload,
            TelemetryFilter,

            // Shared settings models
            NodeSettings,
            MeasurementFrequencies,
            PlantSettings
        )
    ),
    tags(
        (name = "flowers", description = "Plant telemetry and control API")
    )
)]
pub struct ApiDoc;

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct TelemetryFilter {
    pub start: Option<NaiveDateTime>,
    pub end: Option<NaiveDateTime>,
    pub limit: Option<i64>,
}

#[derive(Deserialize, ToSchema)]
pub struct VerboseNamePayload {
    pub verbose_name: String,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub mqtt_client: Arc<AsyncClient>,
}

pub fn app(state: AppState) -> Router {
    Router::new()
        // Docs
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
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
        .route("/api/nodes/{node_id}/settings", post(set_node_settings))
        .route("/api/nodes/{node_id}/channels/{channel_id}/settings", post(set_channel_settings))
        .with_state(state)
}

#[utoipa::path(
    get,
    path = "/api/nodes",
    responses(
        (status = 200, description = "List all nodes", body = [NodeEntry]),
        (status = 500, description = "Database error")
    )
)]
async fn get_all_nodes(State(state): State<AppState>) -> Result<Json<Vec<NodeEntry>>, StatusCode> {
    NodeEntry::get_all(&state.pool)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[utoipa::path(
    get,
    path = "/api/nodes/{node_id}",
    params(
        ("node_id" = String, Path, description = "The ID of the node")
    ),
    responses(
        (status = 200, description = "Node retrieved successfully", body = NodeEntry),
        (status = 404, description = "Node not found"),
        (status = 500, description = "Database error")
    )
)]
async fn get_node_by_id(
    State(state): State<AppState>,
    Path(node_id): Path<String>,
) -> Result<Json<NodeEntry>, StatusCode> {
    match NodeEntry::from_node_id(&state.pool, &node_id).await {
        Ok(Some(node)) => Ok(Json(node)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    get,
    path = "/api/channels",
    responses(
        (status = 200, description = "List all channels", body = [ChannelEntry]),
        (status = 500, description = "Database error")
    )
)]
async fn get_all_channels(State(state): State<AppState>) -> Result<Json<Vec<ChannelEntry>>, StatusCode> {
    ChannelEntry::get_all(&state.pool)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[utoipa::path(
    get,
    path = "/api/nodes/{node_id}/channels/{channel_id}",
    params(
        ("node_id" = String, Path, description = "The ID of the node"),
        ("channel_id" = u8, Path, description = "The ID of the channel")
    ),
    responses(
        (status = 200, description = "Channel retrieved successfully", body = ChannelEntry),
        (status = 404, description = "Channel not found"),
        (status = 500, description = "Database error")
    )
)]
async fn get_channel_by_id(
    State(state): State<AppState>,
    Path((node_id, channel_id)): Path<(String, u8)>,
) -> Result<Json<ChannelEntry>, StatusCode> {
    match ChannelEntry::from_node(&state.pool, &node_id, channel_id).await {
        Ok(Some(channel)) => Ok(Json(channel)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    get,
    path = "/api/nodes/{node_id}/telemetry",
    params(
        ("node_id" = String, Path, description = "The ID of the node"),
        TelemetryFilter
    ),
    responses(
        (status = 200, description = "Node telemetry retrieved successfully", body = [NodeTelemetryEntry]),
        (status = 500, description = "Database error")
    )
)]
async fn get_node_telemetry(
    State(state): State<AppState>,
    Path(node_id): Path<String>,
    Query(filter): Query<TelemetryFilter>,
) -> Result<Json<Vec<NodeTelemetryEntry>>, StatusCode> {
    NodeTelemetryEntry::from_node_id(&state.pool, &node_id, filter.start, filter.end, filter.limit)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[utoipa::path(
    get,
    path = "/api/nodes/{node_id}/channels/{channel_id}/telemetry",
    params(
        ("node_id" = String, Path, description = "The ID of the node"),
        ("channel_id" = u8, Path, description = "The ID of the channel"),
        TelemetryFilter
    ),
    responses(
        (status = 200, description = "Channel telemetry retrieved successfully", body = [ChannelTelemetryEntry]),
        (status = 500, description = "Database error")
    )
)]
async fn get_channel_telemetry(
    State(state): State<AppState>,
    Path((node_id, channel_id)): Path<(String, u8)>,
    Query(filter): Query<TelemetryFilter>,
) -> Result<Json<Vec<ChannelTelemetryEntry>>, StatusCode> {
    ChannelTelemetryEntry::from_index(&state.pool, &node_id, channel_id, filter.start, filter.end, filter.limit)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[utoipa::path(
    post,
    path = "/api/nodes/{node_id}/verbose",
    request_body = VerboseNamePayload,
    params(
        ("node_id" = String, Path, description = "The ID of the node")
    ),
    responses(
        (status = 200, description = "Verbose name updated successfully"),
        (status = 404, description = "Node not found"),
        (status = 500, description = "Database error")
    )
)]
async fn set_node_verbose_name(
    State(state): State<AppState>,
    Path(node_id): Path<String>,
    Json(payload): Json<VerboseNamePayload>,
) -> Result<StatusCode, StatusCode> {
    let mut node = match NodeEntry::from_node_id(&state.pool, &node_id).await {
        Ok(Some(node)) => node,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    match node.set_verbose(&state.pool, payload.verbose_name).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    post,
    path = "/api/nodes/{node_id}/channels/{channel_id}/verbose",
    request_body = VerboseNamePayload,
    params(
        ("node_id" = String, Path, description = "The ID of the node"),
        ("channel_id" = u8, Path, description = "The ID of the channel")
    ),
    responses(
        (status = 200, description = "Channel verbose name updated successfully"),
        (status = 404, description = "Channel not found"),
        (status = 500, description = "Database error")
    )
)]
async fn set_channel_verbose_name(
    State(state): State<AppState>,
    Path((node_id, channel_id)): Path<(String, u8)>,
    Json(payload): Json<VerboseNamePayload>,
) -> Result<StatusCode, StatusCode> {
    let mut channel = match ChannelEntry::from_node(&state.pool, &node_id, channel_id).await {
        Ok(Some(channel)) => channel,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    match channel.set_verbose(&state.pool, payload.verbose_name).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    post,
    path = "/api/nodes/{node_id}/channels/{channel_id}/enable",
    params(
        ("node_id" = String, Path, description = "The ID of the node"),
        ("channel_id" = u8, Path, description = "The ID of the channel")
    ),
    responses(
        (status = 200, description = "Channel enabled successfully"),
        (status = 404, description = "Node or Channel not found"),
        (status = 500, description = "Database error")
    )
)]
async fn enable_channel(
    State(state): State<AppState>,
    Path((node_id, channel_id)): Path<(String, u8)>,
) -> Result<StatusCode, StatusCode> {
    let node = match NodeEntry::from_node_id(&state.pool, &node_id).await {
        Ok(Some(node)) => node,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let mut channel = match ChannelEntry::from_node(&state.pool, &node_id, channel_id).await {
        Ok(Some(channel)) => channel,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    match channel.enable(&state.pool).await {
        Ok(_) => {
            override_settings(state.mqtt_client, &state.pool, node).await;
            Ok(StatusCode::OK)
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    post,
    path = "/api/nodes/{node_id}/channels/{channel_id}/disable",
    params(
        ("node_id" = String, Path, description = "The ID of the node"),
        ("channel_id" = u8, Path, description = "The ID of the channel")
    ),
    responses(
        (status = 200, description = "Channel disabled successfully"),
        (status = 404, description = "Node or Channel not found"),
        (status = 500, description = "Database error")
    )
)]
async fn disable_channel(
    State(state): State<AppState>,
    Path((node_id, channel_id)): Path<(String, u8)>,
) -> Result<StatusCode, StatusCode> {
    let node = match NodeEntry::from_node_id(&state.pool, &node_id).await {
        Ok(Some(node)) => node,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let mut channel = match ChannelEntry::from_node(&state.pool, &node_id, channel_id).await {
        Ok(Some(channel)) => channel,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    match channel.disable(&state.pool).await {
        Ok(_) => {
            override_settings(state.mqtt_client, &state.pool, node).await;
            Ok(StatusCode::OK)
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    post,
    path = "/api/nodes/{node_id}/settings",
    request_body = NodeSettings,
    params(
        ("node_id" = String, Path, description = "The ID of the node")
    ),
    responses(
        (status = 200, description = "Node settings updated successfully"),
        (status = 404, description = "Node not found"),
        (status = 500, description = "Database error")
    )
)]
async fn set_node_settings(
    State(state): State<AppState>,
    Path(node_id): Path<String>,
    Json(settings): Json<NodeSettings>,
) -> Result<StatusCode, StatusCode> {
    let mut node = match NodeEntry::from_node_id(&state.pool, &node_id).await {
        Ok(Some(node)) => node,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    match node.set_settings(&state.pool, settings).await {
        Ok(_) => {
            override_settings(state.mqtt_client, &state.pool, node).await;
            Ok(StatusCode::OK)
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    post,
    path = "/api/nodes/{node_id}/channels/{channel_id}/settings",
    request_body = PlantSettings,
    params(
        ("node_id" = String, Path, description = "The ID of the node"),
        ("channel_id" = u8, Path, description = "The ID of the channel")
    ),
    responses(
        (status = 200, description = "Channel settings updated successfully"),
        (status = 404, description = "Node or Channel not found"),
        (status = 500, description = "Database error")
    )
)]
async fn set_channel_settings(
    State(state): State<AppState>,
    Path((node_id, channel_id)): Path<(String, u8)>,
    Json(settings): Json<PlantSettings>,
) -> Result<StatusCode, StatusCode> {
    let node = match NodeEntry::from_node_id(&state.pool, &node_id).await {
        Ok(Some(node)) => node,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let mut channel = match ChannelEntry::from_node(&state.pool, &node_id, channel_id).await {
        Ok(Some(channel)) => channel, //
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    match channel.set_settings(&state.pool, settings).await {
        Ok(_) => {
            override_settings(state.mqtt_client, &state.pool, node).await;
            Ok(StatusCode::OK)
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}