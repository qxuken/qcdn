use crate::app_state::AppState;
use axum::{http::Method, routing::get, Router};
use tower_http::cors::{self, CorsLayer};

async fn health() -> &'static str {
    "OK"
}

pub fn create_router() -> Router<AppState> {
    let cors = CorsLayer::default()
        .allow_methods([Method::GET])
        .allow_origin(cors::Any);
    Router::new().route("/health", get(health)).layer(cors)
}
