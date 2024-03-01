use std::{os::unix::fs::MetadataExt, path::PathBuf};

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

    let mut file_updates = rpc.connect_to_file_updates().await?;

    let media_type = match media_type {
        Some(t) => t,
        None => {
            let f = FileFormat::from_file(src)?;
            tracing::trace!("{f:?}");
            f.media_type().to_owned()
        }
    };
    tracing::trace!("media_type {media_type}");
    let size: i64 = file.metadata().await?.size().try_into()?;
    tracing::trace!("size {size}");

    let init_message = Ok(upload_request::Request::Meta(UploadMeta {
        name,
        dir,
        size,
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
