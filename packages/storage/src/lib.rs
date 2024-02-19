use std::{
    env,
    fmt::Debug,
    path::{Path, PathBuf},
    sync::Arc,
};

use color_eyre::{eyre::eyre, Report, Result};
use tokio::fs;
use tracing::instrument;

pub use constants::*;

mod constants;

#[derive(Debug, Clone)]
pub struct Storage(Arc<PathBuf>);

impl Storage {
    #[instrument]
    pub async fn try_from_path(path: &Path) -> Result<Self> {
        tracing::trace!("Creating storage");
        let dir = if path.is_absolute() {
            path.into()
        } else {
            env::current_dir()?.join(path)
        };
        tracing::trace!("Checking if {:?} exists", dir);
        if !dir.exists() {
            tracing::debug!("Creating {:?}", dir);
            fs::create_dir(&dir).await?;
        }
        if !dir.is_dir() {
            return Err(eyre!(format!("{dir:?} is not a directory")));
        }

        let storage = Self(Arc::new(dir.to_path_buf()));
        tracing::info!("Created storage");
        tracing::trace!("{:?}", storage);
        Ok(storage)
    }
}

impl Storage {
    #[instrument(skip(self))]
    pub fn get_path(&self, relative_path: &str) -> PathBuf {
        let path = self.0.clone().join(relative_path);
        tracing::trace!("Storage path {path:?}");
        path
    }

    #[instrument(skip(self))]
    pub async fn open_file(&self, dir: &str, filename: &str) -> Result<fs::File> {
        let path = self.0.clone().join(dir).join(filename);
        tracing::trace!("Opening file {path:?}");

        Ok(fs::File::open(path).await?)
    }

    #[instrument(skip(self))]
    pub async fn create_file(&self, dir: &str, filename: &str) -> Result<fs::File> {
        let dir_path = self.0.clone().join(dir);
        tracing::trace!("Checking directory {dir_path:?}");
        if fs::read_dir(&dir_path).await.is_err() {
            tracing::trace!("Creating directory {dir_path:?}");
            fs::create_dir(&dir_path).await?;
        }
        let file_path = dir_path.join(filename);
        tracing::trace!("Creating file {file_path:?}");
        Ok(fs::File::create(file_path).await?)
    }

    #[instrument(skip(self))]
    pub async fn remove_file(&self, dir: &str, filename: &str) -> Result<()> {
        let file_path = self.0.clone().join(dir).join(filename);
        if !file_path.is_file() {
            return Err(Report::msg(format!("{file_path:?} is not a file")));
        }
        tracing::trace!("Removing file {file_path:?}");
        Ok(fs::remove_file(file_path).await?)
    }
}
