use std::path::PathBuf;

use color_eyre::{eyre::eyre, Result};
use tokio::{fs, io::AsyncWriteExt};
use tokio_stream::StreamExt;

use crate::{cli::Cli, rpc::Rpc};

#[tracing::instrument(skip_all)]
pub async fn download(cli: &Cli, file_version_id: i64, path: Option<PathBuf>) -> Result<()> {
    let rpc: Rpc = cli.into();

    let mut file_query = rpc.connect_to_file_query().await?;
    let mut stream = Rpc::get_download_stream(&mut file_query, file_version_id).await?;

    match path.as_ref() {
        Some(path) => {
            if let Some(parent) = path.parent() {
                tracing::trace!("Ensure dir exists {parent:?}");
                fs::create_dir_all(parent).await?;
            }
            tracing::trace!("Opening file {path:?}");
            let mut file = fs::File::create(path).await?;
            while let Some(part) = stream.next().await.transpose()? {
                file.write_all(&part.bytes).await?;
            }
        }
        None => {
            let mut buf = vec![];
            while let Some(part) = stream.next().await.transpose()? {
                buf.extend(part.bytes);
            }
            print!("{}", String::from_utf8_lossy(&buf));
        }
    }
    if let Some(path) = path.as_ref() {
        tracing::trace!("Verifying hash");
        let version = Rpc::get_version(&mut file_query, file_version_id).await?;

        let hash = qcdn_utils::hash::sha256_file(path).await?;
        if version.hash != hash {
            return Err(eyre!(
                "File hash does not match, transmission must be corrupted"
            ));
        }
    }

    println!("Ok");

    Ok(())
}
