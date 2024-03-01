use color_eyre::Result;

use crate::rpc::Rpc;

#[tracing::instrument(skip_all)]
pub async fn handshake_server(rpc: &Rpc) -> Result<()> {
    let client_version = Rpc::client_version();
    let general = rpc.connect_to_general().await?;
    let (latency, server_version) = tokio::try_join!(
        async {
            let mut general = general.clone();
            Rpc::latency(&mut general).await
        },
        async {
            let mut general = general.clone();
            Rpc::server_version(&mut general).await
        }
    )?;

    if let Some(latency) = latency {
        println!("Latency to server is {}", latency);
    }

    if client_version != server_version {
        println!(
            "Version mismatch!\nServer version is {}, client version is {}",
            server_version,
            Rpc::client_version()
        );
    } else {
        println!("Server version is {}", server_version,);
    }

    Ok(())
}
