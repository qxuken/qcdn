use color_eyre::Result;

use crate::{cli::Cli, rpc::Rpc};

#[tracing::instrument(skip_all)]
pub async fn tag_version(cli: &Cli, file_version_id: i64, tag: String) -> Result<()> {
    let rpc: Rpc = cli.into();

    let mut file_updates = rpc.connect_to_file_updates().await?;
    Rpc::tag_version(&mut file_updates, file_version_id, tag).await?;

    println!("Ok");

    Ok(())
}
