#![feature(io_error_more)]
use std::{
    env,
    fmt::Debug,
    io::Error,
    path::{Path, PathBuf},
    sync::Arc,
};

use color_eyre::Result;
use tokio::fs;
use tracing::instrument;

pub use constants::*;

mod constants;

#[derive(Debug, Clone)]
pub struct Storage {
    root: Arc<PathBuf>,
    sub_dir: Arc<PathBuf>,
}

impl Storage {
    #[instrument]
    pub async fn try_from_path(path: &Path, subdir: &'static str) -> Result<Self, Error> {
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
            return Err(Error::new(
                std::io::ErrorKind::NotADirectory,
                format!("{target:?} is not a directory"),
            ));
        }

        let storage = Self {
            root: Arc::new(dir.to_path_buf()),
            sub_dir: Arc::new(target),
        };
        tracing::info!("Created storage");
        tracing::trace!("{:?}", storage);
        Ok(storage)
    }
}

impl Storage {
    #[instrument(skip(self))]
    pub fn ping(&self) -> Result<(), Error> {
        if !self.sub_dir.is_dir() {
            return Err(Error::new(
                std::io::ErrorKind::NotFound,
                "Storage sub dir is not exists",
            ));
        }
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn get_path<P: AsRef<Path> + Debug>(&self, relative_path: P, from_root: bool) -> PathBuf {
        let dir = if from_root {
            self.root.clone()
        } else {
            self.sub_dir.clone()
        };
        let path = dir.join(relative_path);
        tracing::trace!("Storage path {path:?}");
        path
    }

    #[instrument(skip(self))]
    pub async fn open_file<P: AsRef<Path> + Debug>(
        &self,
        relative_path: P,
    ) -> Result<fs::File, Error> {
        let path = self.sub_dir.join(relative_path);
        tracing::trace!("Opening file {path:?}");

        fs::File::open(path).await
    }

    #[instrument(skip(self))]
    pub async fn create_file<P: AsRef<Path> + Debug>(
        &self,
        relative_path: P,
    ) -> Result<fs::File, Error> {
        let file_path = self.sub_dir.join(relative_path);
        let dir_path = file_path
            .parent()
            .ok_or_else(|| Error::new(std::io::ErrorKind::NotFound, "Unable to find parent dir"))?;
        tracing::trace!("Checking directory {dir_path:?}");
        if fs::read_dir(&dir_path).await.is_err() {
            tracing::trace!("Creating directory {dir_path:?}");
            fs::create_dir_all(&dir_path).await?;
        }
        tracing::trace!("Creating file {file_path:?}");
        fs::File::create(file_path).await
    }

    #[instrument(skip(self))]
    pub async fn remove_file<P: AsRef<Path> + Debug>(&self, relative_path: P) -> Result<(), Error> {
        let file_path = self.sub_dir.join(relative_path);
        if !file_path.is_file() {
            return Err(Error::new(
                std::io::ErrorKind::NotFound,
                format!("{file_path:?} is not a file"),
            ));
        }
        tracing::trace!("Removing file {file_path:?}");
        fs::remove_file(file_path).await
    }
}
