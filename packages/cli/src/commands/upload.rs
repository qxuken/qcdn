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
    let hash = qcdn_utils::hash::sha256_file(&src).await?;

    let file = fs::File::open(&src).await?;
    let size = ReaderStream::new(file)
        .fold(0, |size, bytes| async move {
            size + bytes.map(|b| b.len()).unwrap_or_default()
        })
        .await
        .try_into()?;
    tracing::trace!("size {size}");

    let format_src = src.clone();
    let media_type = tokio::spawn(async move {
        media_type
            .map(Ok)
            .unwrap_or_else(|| FileFormat::from_file(format_src).map(|f| f.media_type().to_owned()))
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

    let file = fs::File::open(&src).await?;
    let file_stream = ReaderStream::new(file).map(|frame| {
        frame
            .inspect(|bytes| tracing::trace!("Sending {} bytes", bytes.len()))
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
