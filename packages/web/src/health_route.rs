use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use tracing::instrument;

use crate::{app_state::SharedAppState, error::Result};

#[instrument]
pub async fn health_route(State(app): State<SharedAppState>) -> Result<Response<Body>> {
    tracing::info!("Got request");
    app.db.establish_connection().await?;
    app.storage.ping()?;

    Ok((StatusCode::OK, "Ok").into_response())
}
