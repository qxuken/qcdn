use color_eyre::Result;

use crate::{rpc::Rpc, utils::std_table::StdTable};

#[tracing::instrument(skip_all)]
pub async fn list_dirs(rpc: &Rpc) -> Result<()> {
    let mut file_query = rpc.connect_to_file_query().await?;
    let dirs = Rpc::list_dirs(&mut file_query).await?;

    let mut table = StdTable::new(vec!["id", "name"]);

    for dir in dirs.into_iter() {
        table.add_row(vec![dir.id.to_string(), dir.name]);
    }
    println!("{table}");

    Ok(())
}
