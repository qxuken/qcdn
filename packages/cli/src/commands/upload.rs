use filesize::PathExt;
use std::path::PathBuf;

use crate::{cli::Cli, rpc::Rpc};
use color_eyre::Result;
use file_format::FileFormat;
use futures::StreamExt;
use qcdn_proto_client::{upload_request, FilePart, UploadMeta, UploadRequest};
use tokio::fs;
use tokio_util::io::ReaderStream;

#[tracing::instrument(skip(cli))]
pub async fn upload(
    cli: &Cli,
    dir: String,
    name: String,
    version: String,
    media_type: Option<String>,
    src: PathBuf,
) -> Result<()> {
    let rpc: Rpc = cli.into();

    tracing::debug!("Opening file handle");
    let file = fs::File::open(&src).await?;
    let hash = qcdn_utils::hash::sha256_file(&src).await?;

    let size: i64 = src.as_path().size_on_disk()?.try_into()?;
    tracing::trace!("size {size}");

    let media_type = tokio::spawn(async {
        media_type
            .map(Ok)
            .unwrap_or_else(|| FileFormat::from_file(src).map(|f| f.media_type().to_owned()))
    })
    .await??;
    tracing::trace!("media_type {media_type}");

    let mut file_updates = rpc.connect_to_file_updates().await?;

    let init_message = Ok(upload_request::Request::Meta(UploadMeta {
        name,
        dir,
        size,
        hash,
        media_type,
        version,
    }));

    let file_stream = ReaderStream::new(file).map(|frame| {
        frame
            .map(|bytes| FilePart {
                bytes: bytes.into(),
            })
            .map(upload_request::Request::Part)
    });

    let stream = tokio_stream::once(init_message)
        .chain(file_stream)
        .map(|req| UploadRequest { request: req.ok() });

    tracing::debug!("Starting upload");
    let uploaded_file = file_updates.upload(stream).await?.into_inner();

    tracing::info!("Uploaded file");

    println!("{uploaded_file:?}");
    println!("Ok");
    Ok(())
}
