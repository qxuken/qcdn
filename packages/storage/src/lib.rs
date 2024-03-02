use std::{
    env,
    fmt::Debug,
    path::{Path, PathBuf},
    sync::Arc,
};

use color_eyre::{
    eyre::{eyre, OptionExt},
    Report, Result,
};
use tokio::fs;
use tracing::instrument;

pub use constants::*;

mod constants;

#[derive(Debug, Clone)]
pub struct Storage {
    root: Arc<PathBuf>,
    subdir: Arc<PathBuf>,
}

impl Storage {
    #[instrument]
    pub async fn try_from_path(path: &Path, subdir: &'static str) -> Result<Self> {
        tracing::trace!("Creating storage");
        let dir = if path.is_absolute() {
            path.into()
        } else {
            env::current_dir()?.join(path)
        };
        let target = dir.join(subdir);
        tracing::trace!("Checking if {:?} exists", target);
        if !target.exists() {
            tracing::debug!("Creating {:?}", target);
            fs::create_dir_all(&target).await?;
        }
        if !target.is_dir() {
            return Err(eyre!(format!("{target:?} is not a directory")));
        }

        let storage = Self {
            root: Arc::new(dir.to_path_buf()),
            subdir: Arc::new(target),
        };
        tracing::info!("Created storage");
        tracing::trace!("{:?}", storage);
        Ok(storage)
    }
}

impl Storage {
    #[instrument(skip(self))]
    pub fn get_path<P: AsRef<Path> + Debug>(&self, relative_path: P, from_root: bool) -> PathBuf {
        let dir = if from_root {
            self.root.clone()
        } else {
            self.subdir.clone()
        };
        let path = dir.join(relative_path);
        tracing::trace!("Storage path {path:?}");
        path
    }

    #[instrument(skip(self))]
    pub async fn open_file<P: AsRef<Path> + Debug>(&self, relative_path: P) -> Result<fs::File> {
        let path = self.subdir.join(relative_path);
        tracing::trace!("Opening file {path:?}");

        Ok(fs::File::open(path).await?)
    }

    #[instrument(skip(self))]
    pub async fn create_file<P: AsRef<Path> + Debug>(&self, relative_path: P) -> Result<fs::File> {
        let file_path = self.subdir.join(relative_path);
        let dir_path = file_path.parent().ok_or_eyre("Unable to find parent dir")?;
        tracing::trace!("Checking directory {dir_path:?}");
        if fs::read_dir(&dir_path).await.is_err() {
            tracing::trace!("Creating directory {dir_path:?}");
            fs::create_dir_all(&dir_path).await?;
        }
        tracing::trace!("Creating file {file_path:?}");
        Ok(fs::File::create(file_path).await?)
    }

    #[instrument(skip(self))]
    pub async fn remove_file<P: AsRef<Path> + Debug>(&self, relative_path: P) -> Result<()> {
        let file_path = self.subdir.join(relative_path);
        if !file_path.is_file() {
            return Err(Report::msg(format!("{file_path:?} is not a file")));
        }
        tracing::trace!("Removing file {file_path:?}");
        Ok(fs::remove_file(file_path).await?)
    }
}
