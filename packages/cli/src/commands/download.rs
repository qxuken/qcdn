use std::path::PathBuf;

use color_eyre::Result;
use tokio::{fs, io::AsyncWriteExt};
use tokio_stream::StreamExt;

use crate::{cli::Cli, rpc::Rpc};

#[tracing::instrument(skip_all)]
pub async fn download(cli: &Cli, file_version_id: i64, path: Option<PathBuf>) -> Result<()> {
    let rpc: Rpc = cli.into();

    let mut file_query = rpc.connect_to_file_query().await?;
    let mut stream = Rpc::get_download_stream(&mut file_query, file_version_id).await?;

    match path {
        Some(path) => {
            if let Some(parent) = path.parent() {
                tracing::trace!("Ensure dir exists {parent:?}");
                fs::create_dir_all(parent).await?;
            }
            tracing::trace!("Openning file {path:?}");
            let mut file = fs::File::create(path).await?;
            while let Some(part) = stream.next().await.transpose()? {
                file.write_all(&part.bytes).await?;
            }
            println!("Ok");
        }
        None => {
            let mut buf = vec![];
            while let Some(part) = stream.next().await.transpose()? {
                buf.extend(part.bytes);
            }
            print!("{}", String::from_utf8_lossy(&buf));
        }
    }

    Ok(())
}
