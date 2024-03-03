#![feature(try_blocks)]

use std::sync::Arc;

use app_state::AppState;
use axum::{http::StatusCode, routing::get, Router};
use clap::Parser;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing::instrument;

pub mod app_state;
mod cli;
pub mod constants;
pub mod error;
pub mod file_route;
pub mod health_route;
mod trace_layer;

#[instrument]
async fn fallback() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not Found")
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    dotenvy::dotenv().ok();

    qcdn_utils::color_eyre::setup()?;

    let cli = cli::Cli::parse();
    cli.instrumentation.setup(&[
        constants::PACKAGE_NAME,
        qcdn_utils::PACKAGE_NAME,
        qcdn_database::PACKAGE_NAME,
        qcdn_storage::PACKAGE_NAME,
    ])?;

    let storage = Arc::new(qcdn_storage::Storage::try_from_path(&cli.data, "storage").await?);

    let db_path = storage.get_path(qcdn_database::DB_NAME, true);
    let db = Arc::new(qcdn_database::Database::try_from_path(&db_path, true).await?);

    let state = Arc::new(AppState::new(
        storage,
        db,
        matches!(cli.mode, cli::Mode::Development),
    ));

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(trace_layer::trace_layer_make_span_with)
        .on_request(trace_layer::trace_layer_on_request)
        .on_response(trace_layer::trace_layer_on_response);
    let compression = CompressionLayer::new();
    let app = Router::new()
        .route("/health", get(health_route::health_route))
        .route("/f/:dir/:file/:version_or_tag", get(file_route::file_route))
        .fallback(fallback)
        .with_state(state)
        .layer(compression)
        .layer(trace_layer);

    tracing::info!("Listening on {}", cli.bind);
    let listener = tokio::net::TcpListener::bind(cli.bind).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
