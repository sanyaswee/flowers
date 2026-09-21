//! All API handlers

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;

use chrono::NaiveDateTime;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use shared::node_settings::{NodeSettings, PlantSettings};

use crate::api::AppState;
use crate::db::entries::{ChannelEntry, ChannelTelemetryEntry, NodeEntry, NodeTelemetryEntry};
use crate::mqtt::router::{override_settings, water_plant};

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

#[utoipa::path(
    get,
    path = "/api/nodes",
    responses(
        (status = 200, description = "List all nodes", body = [NodeEntry]),
        (status = 500, description = "Database error")
    )
)]
pub async fn get_all_nodes(
    State(state): State<AppState>,
) -> Result<Json<Vec<NodeEntry>>, StatusCode> {
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
pub async fn get_node_by_id(
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
pub async fn get_all_channels(
    State(state): State<AppState>,
) -> Result<Json<Vec<ChannelEntry>>, StatusCode> {
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
pub async fn get_channel_by_id(
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
pub async fn get_node_telemetry(
    State(state): State<AppState>,
    Path(node_id): Path<String>,
    Query(filter): Query<TelemetryFilter>,
) -> Result<Json<Vec<NodeTelemetryEntry>>, StatusCode> {
    NodeTelemetryEntry::from_node_id(
        &state.pool,
        &node_id,
        filter.start,
        filter.end,
        filter.limit,
    )
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
pub async fn get_channel_telemetry(
    State(state): State<AppState>,
    Path((node_id, channel_id)): Path<(String, u8)>,
    Query(filter): Query<TelemetryFilter>,
) -> Result<Json<Vec<ChannelTelemetryEntry>>, StatusCode> {
    ChannelTelemetryEntry::from_index(
        &state.pool,
        &node_id,
        channel_id,
        filter.start,
        filter.end,
        filter.limit,
    )
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
pub async fn set_node_verbose_name(
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
pub async fn set_channel_verbose_name(
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
pub async fn enable_channel(
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
        }
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
pub async fn disable_channel(
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
        }
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
pub async fn set_node_settings(
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
        }
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
pub async fn set_channel_settings(
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
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[utoipa::path(
    post,
    path = "/api/nodes/{node_id}/channels/{channel_id}/water",
    params(
        ("node_id" = String, Path, description = "The ID of the node"),
        ("channel_id" = u8, Path, description = "The ID of the channel")
    ),
    responses(
        (status = 200, description = "Watering command sent successfully"),
        (status = 404, description = "Node or channel not found"),
        (status = 500, description = "Database error")
    )
)]
pub async fn water_channel(
    State(state): State<AppState>,
    Path((node_id, channel_id)): Path<(String, u8)>,
) -> Result<StatusCode, StatusCode> {
    let node = match NodeEntry::from_node_id(&state.pool, &node_id).await {
        Ok(Some(node)) => node,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Verify the channel actually exists in the database
    let _channel = match ChannelEntry::from_node(&state.pool, &node_id, channel_id).await {
        Ok(Some(channel)) => channel,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    water_plant(state.mqtt_client, node, channel_id).await;

    Ok(StatusCode::OK)
}
