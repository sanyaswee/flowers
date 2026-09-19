//! This module is responsible for API endpoints

mod handlers;

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};

use rumqttc::AsyncClient;
use sqlx::SqlitePool;

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use shared::node_settings::{MeasurementFrequencies, NodeSettings, PlantSettings};

use crate::db::entries::*;
use handlers::*;

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
