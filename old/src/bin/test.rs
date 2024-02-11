use qcdn::{
    config::CliConfig,
    grpc::{
        qcdn_files_client::QcdnFilesClient, qcdn_general_client::QcdnGeneralClient, upload_request,
        DeleteFileVersionRequest, DownloadRequest, FilePart, FileType, GetFileVersionsRequest,
        GetFilesRequest, PingMessage, UploadMeta, UploadRequest,
    },
    setup_tracing_subscriber,
};
use std::time::SystemTime;
use tokio_stream::StreamExt;
use tonic::Request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let config = CliConfig::init();

    setup_tracing_subscriber(config.log_level);

    tracing::info!("{:?}", config);

    let addr = config
        .main_server_url
        .expect("Master address must be present");

    let mut general = QcdnGeneralClient::connect(addr.clone()).await?;

    let ping = PingMessage {
        timestamp: Some(SystemTime::now().into()),
    };
    let response = general.ping(Request::new(ping)).await?.into_inner();

    tracing::info!("{response:?}");

    let response = general.version(Request::new(())).await?.into_inner();

    tracing::info!("{response:?}");

    let mut files = QcdnFilesClient::connect(addr).await?;

    let response = files
        .get_files(Request::new(GetFilesRequest { dir_id: None }))
        .await?
        .into_inner();

    tracing::info!("{response:?}");

    let test_file: &[u8] = include_bytes!("../../data/input/test.txt");
    let init_message = upload_request::Request::Meta(UploadMeta {
        name: "test".to_string(),
        dir: "test".to_string(),
        size: test_file.len() as u64,
        file_type: FileType::Text.into(),
        version: "1".to_string(),
    });
    let chunked = tokio_stream::iter(test_file.chunks(4096))
        .map(|bytes| FilePart {
            bytes: bytes.to_vec(),
        })
        .map(upload_request::Request::Part);

    let stream = tokio_stream::once(init_message)
        .chain(chunked)
        .map(|req| UploadRequest { request: Some(req) });

    let uploaded_file = files.upload(stream).await?.into_inner();

    tracing::info!("{uploaded_file:?}");

    let file_id = uploaded_file.file_id;
    let file_version_id = uploaded_file.file_version_id;

    let response = files
        .get_file_versions(Request::new(GetFileVersionsRequest {
            file_id: file_id.to_owned(),
        }))
        .await?
        .into_inner();

    tracing::info!("{response:?}");

    files
        .delete_file_version(Request::new(DeleteFileVersionRequest {
            id: file_version_id.to_owned(),
        }))
        .await?
        .into_inner();

    tracing::info!("deleted file_version_id: {file_version_id:?}");

    let response = files
        .get_file_versions(Request::new(GetFileVersionsRequest {
            file_id: file_id.to_owned(),
        }))
        .await?
        .into_inner();

    tracing::info!("{response:?}");

    let mut response = files
        .download(Request::new(DownloadRequest {
            file_version_id: file_version_id.to_owned(),
        }))
        .await?
        .into_inner();

    while let Some(part) = response.next().await.transpose()? {
        println!("Got part {:?}", part.bytes.len());
    }

    Ok(())
}
