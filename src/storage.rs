use std::{
    fmt::Debug,
    path::{Path, PathBuf},
    sync::Arc,
};

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};
use tokio::fs;

use crate::AppState;

#[derive(Debug, Clone)]
pub struct Storage(Arc<PathBuf>);

impl Storage {
    pub async fn from_path(value: &Path) -> Result<Self, anyhow::Error> {
        tracing::debug!("Checking if {:?} exists", value);
        if !value.exists() {
            tracing::debug!("Creating {:?}", value);
            fs::create_dir(&value)
                .await
                .map_err(|e| anyhow::anyhow!("{:?}: {}", value, e))?;
        }
        if !value.is_dir() {
            return Err(anyhow::anyhow!("{:?} is not a directory", value));
        }

        let storage = Self(Arc::new(value.to_path_buf()));
        tracing::debug!("{:?} created", storage);
        Ok(storage)
    }
}

impl FromRef<AppState> for Storage {
    fn from_ref(app_state: &AppState) -> Storage {
        app_state.storage.clone()
    }
}

#[async_trait]
impl FromRequestParts<AppState> for Storage
where
    Storage: FromRef<AppState>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let storage = Storage::from_ref(state);
        Ok(storage)
    }
}
