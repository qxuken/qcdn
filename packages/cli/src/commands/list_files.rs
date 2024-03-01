use color_eyre::Result;

use crate::{rpc::Rpc, utils::std_table::StdTable};

#[tracing::instrument(skip_all)]
pub async fn list_files(rpc: &Rpc, dir_id: i64) -> Result<()> {
    let mut file_query = rpc.connect_to_file_query().await?;
    let files = Rpc::list_files(&mut file_query, dir_id).await?;

    let mut table = StdTable::new(vec!["id", "type", "name"]);

    for file in files.into_iter() {
        table.add_row(vec![
            file.id.to_string(),
            format!("{:?}", file.file_type()),
            file.name,
        ]);
    }

    println!("{table}");

    Ok(())
}
