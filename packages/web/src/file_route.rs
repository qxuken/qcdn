use axum::{
    body::Body,
    extract::{Path, State},
    http::{
        header::{
            CACHE_CONTROL, CONTENT_TYPE, ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED,
        },
        HeaderMap, StatusCode,
    },
    response::{IntoResponse, Response},
};
use tokio_util::io::ReaderStream;
use tracing::instrument;

use qcdn_database::FileVersionMeta;

use crate::{
    app_state::SharedAppState,
    error::{AppError, Result},
};

#[axum_macros::debug_handler]
#[instrument(skip(state))]
pub async fn file_route(
    headers: HeaderMap,
    Path((dir, file, version_or_tag)): Path<(String, String, String)>,
    State(state): State<SharedAppState>,
) -> Result<Response<Body>> {
    let mut connection = state
        .db
        .establish_connection()
        .await
        .map_err(AppError::from)?;
    let meta = FileVersionMeta::find(&mut connection, &dir, &file, &version_or_tag)
        .await
        .map_err(AppError::from)?;

    let if_none_match = headers
        .get(IF_NONE_MATCH)
        .and_then(|h| h.to_str().ok())
        .map(|v| v.to_owned());
    let if_modified_since = headers
        .get(IF_MODIFIED_SINCE)
        .and_then(|h| h.to_str().ok())
        .map(|v| v.to_owned());

    let mut headers = HeaderMap::with_capacity(4);

    if if_none_match.is_some_and(|v| v.contains(&meta.hash)) {
        return Ok(StatusCode::NOT_MODIFIED.into_response());
    }
    headers.insert(ETAG, meta.hash.parse().unwrap());

    let last_modified = meta.created_at.format("%a, %e %b %T %Y %T GMT").to_string();
    if if_modified_since.is_some_and(|v| v == last_modified) {
        return Ok(StatusCode::NOT_MODIFIED.into_response());
    }
    headers.insert(LAST_MODIFIED, last_modified.parse().unwrap());

    let mime = meta.media_type;
    if state.is_dev {
        headers.insert(CACHE_CONTROL, "private, no-cache".parse().unwrap());
    } else {
        headers.insert(
            CACHE_CONTROL,
            "public, max-age=31536000, immutable".parse().unwrap(),
        );
    }
    headers.insert(CONTENT_TYPE, mime.parse().unwrap());

    let file = state.storage.open_file(&meta.path).await?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Ok((headers, body).into_response())
}
