use color_eyre::Result;

use crate::{
    cli::Cli,
    rpc::Rpc,
    utils::std_table::{Format, StdTable},
};

#[tracing::instrument(skip_all)]
pub async fn list_files(cli: &Cli, dir_id: i64, format: Format) -> Result<()> {
    let rpc: Rpc = cli.into();

    let mut file_query = rpc.connect_to_file_query().await?;
    let files = Rpc::list_files(&mut file_query, dir_id).await?;

    let mut table = StdTable::new(vec!["id", "media type", "name"], format);

    for file in files.into_iter() {
        table.add_row(vec![file.id.to_string(), file.media_type, file.name]);
    }

    println!("{table}");

    Ok(())
}
