use std::time::SystemTime;

use color_eyre::Result;
use qcdn_proto_client::{
    qcdn_file_queries_client::QcdnFileQueriesClient,
    qcdn_file_updates_client::QcdnFileUpdatesClient, qcdn_general_client::QcdnGeneralClient,
    upload_request, DeleteFileVersionRequest, DownloadRequest, FilePart, FileType,
    GetFileVersionsRequest, GetFilesRequest, PingMessage, UploadMeta, UploadRequest,
};
use tokio_stream::StreamExt;
use tonic::Request;

use crate::cli::Cli;

#[tracing::instrument]
pub(crate) async fn upload(cli: &Cli, version: &str, save_version: &bool) -> Result<()> {
    tracing::info!("Connecting");
    let mut general = QcdnGeneralClient::connect(cli.url.clone()).await?;

    let ts = SystemTime::now();
    tracing::info!("Sending ping at {ts:?}");
    let ping = PingMessage {
        timestamp: Some(ts.into()),
    };
    let response = general.ping(Request::new(ping)).await?.into_inner();

    tracing::info!("{response:?}");

    let response = general.version(Request::new(())).await?.into_inner();

    tracing::info!("{response:?}");

    let mut file_queries = QcdnFileQueriesClient::connect(cli.url.clone()).await?;
    let mut file_updates = QcdnFileUpdatesClient::connect(cli.url.clone()).await?;

    tracing::info!("Dirs");
    let response = file_queries.get_dirs(Request::new(())).await?.into_inner();
    tracing::info!("{response:?}");

    if let Some(dir) = response.items.first() {
        tracing::info!("Dir({}) Files:", dir.id);
        let response = file_queries
            .get_files(Request::new(GetFilesRequest { dir_id: dir.id }))
            .await?
            .into_inner();
        tracing::info!("{response:?}");

        if let Some(file) = response.items.first() {
            tracing::info!("File({}) versions:", file.id);
            let response = file_queries
                .get_file_versions(Request::new(GetFileVersionsRequest { file_id: file.id }))
                .await?
                .into_inner();
            tracing::info!("{response:?}");
        }
    }

    let test_file: &[u8] = include_bytes!("../../../data/input/test.txt");
    let init_message = upload_request::Request::Meta(UploadMeta {
        name: "test".to_string(),
        dir: "test".to_string(),
        size: test_file.len().try_into()?,
        file_type: FileType::Text.into(),
        version: version.to_string(),
    });
    let chunked = tokio_stream::iter(test_file.chunks(4096))
        .map(|bytes| FilePart {
            bytes: bytes.to_vec(),
        })
        .map(upload_request::Request::Part);

    let stream = tokio_stream::once(init_message)
        .chain(chunked)
        .map(|req| UploadRequest { request: Some(req) });

    tracing::info!("Starting upload");
    let uploaded_file = file_updates.upload(stream).await?.into_inner();

    tracing::info!("Uploaded file");
    tracing::info!("{uploaded_file:?}");

    let file_id = uploaded_file.file_id;
    let file_version_id = uploaded_file.file_version_id;

    tracing::info!("File({file_id}) versions:");
    let response = file_queries
        .get_file_versions(Request::new(GetFileVersionsRequest {
            file_id: file_id.to_owned(),
        }))
        .await?
        .into_inner();
    tracing::info!("{response:?}");

    if !save_version {
        file_updates
            .delete_file_version(Request::new(DeleteFileVersionRequest {
                id: file_version_id.to_owned(),
            }))
            .await?
            .into_inner();
        tracing::info!("Deleted FileVersion({file_version_id})");
    }

    tracing::info!("File({file_id}) versions:");
    let response = file_queries
        .get_file_versions(Request::new(GetFileVersionsRequest {
            file_id: file_id.to_owned(),
        }))
        .await?
        .into_inner();
    tracing::info!("{response:?}");

    tracing::info!("Downloading FileVersion({file_version_id})");
    let mut response = file_queries
        .download(Request::new(DownloadRequest {
            file_version_id: file_version_id.to_owned(),
        }))
        .await?
        .into_inner();

    while let Some(part) = response.next().await.transpose()? {
        println!("Got part {:?}", part.bytes.len());
    }
    tracing::info!("Done");

    Ok(())
}
