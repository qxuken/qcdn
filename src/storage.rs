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
    pub async fn open_file(&self, dir: &str, filename: &str) -> Result<fs::File, anyhow::Error> {
        let path = self.0.clone().join(dir).join(filename);

        Ok(fs::File::open(path).await?)
    }

    pub async fn create_file(&self, dir: &str, filename: &str) -> Result<fs::File, anyhow::Error> {
        let dir_path = self.0.clone().join(dir);
        if fs::read_dir(&dir_path).await.is_err() {
            fs::create_dir(&dir_path).await?;
        }
        Ok(fs::File::create(dir_path.join(filename)).await?)
    }

    pub async fn remove_file(&self, dir: &str, filename: &str) -> Result<(), anyhow::Error> {
        let dir_path = self.0.clone().join(dir);
        Ok(fs::remove_file(dir_path.join(filename)).await?)
    }
}

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
