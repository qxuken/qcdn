pub mod http;

use crate::app_state::AppState;
use axum::Router;
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

pub fn create_router() -> Router<AppState> {
    let trace = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new())
        .on_response(
            DefaultOnResponse::new()
                .level(Level::INFO)
                .latency_unit(LatencyUnit::Micros),
        );
    let http_router = http::create_router().layer(trace);

    Router::new().merge(http_router)
}
