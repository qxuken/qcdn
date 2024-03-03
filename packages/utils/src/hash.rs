use color_eyre::Result;
use data_encoding::BASE64;
use ring::digest::{Context, SHA256};
use std::{fmt::Debug, io::Error, path::Path};
use tokio::fs;
use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;
use tracing::instrument;

#[instrument]
pub async fn sha256_file<P: AsRef<Path> + Debug>(path: P) -> Result<String, Error> {
    tracing::trace!("Opening file handle");
    let file = fs::File::open(path).await?;

    let mut file_stream = ReaderStream::new(file);

    let mut context = Context::new(&SHA256);

    tracing::trace!("Reading file stream into hash context");
    while let Some(chunk) = file_stream.next().await.transpose()? {
        context.update(&chunk);
    }

    let dig = context.finish();
    let hash = BASE64.encode(dig.as_ref());

    tracing::trace!("Hash {hash}");
    Ok(hash)
}
