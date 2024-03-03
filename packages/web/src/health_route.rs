use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::instrument;

use crate::app_state::SharedAppState;

#[instrument]
pub async fn health_route(State(app): State<SharedAppState>) -> Response {
    tracing::info!("Got request");
    if let Err(e) = app.db.establish_connection().await {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }
    if let Err(e) = app.storage.ping() {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    (StatusCode::OK, "Ok").into_response()
}
