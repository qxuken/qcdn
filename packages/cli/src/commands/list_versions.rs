use color_eyre::Result;

use crate::{rpc::Rpc, utils::std_table::StdTable};

#[tracing::instrument(skip_all)]
pub async fn list_versions(rpc: &Rpc, file_id: i64) -> Result<()> {
    let mut file_query = rpc.connect_to_file_query().await?;
    let file_versions = Rpc::list_versions(&mut file_query, file_id).await?;

    let mut table = StdTable::new(vec!["id", "size, B", "version", "deleted", "tags"]);

    for v in file_versions.into_iter() {
        let mut row = vec![
            v.id.to_string(),
            v.size.to_string(),
            v.version,
            if v.is_deleted {
                "active".to_string()
            } else {
                "deleted".to_string()
            },
        ];
        if let Some(tags) = v.tags.into_iter().reduce(|mut r, t| {
            r.push_str(&format!(", {}", t));
            r
        }) {
            row.push(tags);
        }
        table.add_row(row);
    }

    println!("{table}");

    Ok(())
}
