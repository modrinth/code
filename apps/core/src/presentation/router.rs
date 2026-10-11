use std::sync::Arc;

use axum::{
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::{application::state::AppState, presentation::handlers::pairing};

/// Build the Axum router: the hosting API, pairing, and a liveness probe.
pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = cors_layer(&state);

    Router::new()
        .nest("/hosting", crate::presentation::hosting::router())
        .route("/pairing", post(pairing::pair).delete(pairing::unpair))
        .route("/health", get(health))
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

fn cors_layer(state: &AppState) -> CorsLayer {
    if state.config.dev_mode || state.config.allowed_origin == "*" {
        return CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);
    }

    let origin = state
        .config
        .allowed_origin
        .parse::<HeaderValue>()
        .expect("ALLOWED_ORIGIN must be '*' or a valid HTTP origin");

    CorsLayer::new()
        .allow_origin(origin)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers([
            AUTHORIZATION,
            CONTENT_TYPE,
            axum::http::HeaderName::from_static("x-panel-version"),
            axum::http::HeaderName::from_static("last-event-id"),
        ])
        .allow_credentials(true)
}
