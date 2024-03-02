use color_eyre::Result;

use crate::{
    cli::Cli,
    rpc::Rpc,
    utils::std_table::{Format, StdTable},
};

#[tracing::instrument(skip_all)]
pub async fn list_versions(cli: &Cli, file_id: i64, format: Format) -> Result<()> {
    let rpc: Rpc = cli.into();

    let mut file_query = rpc.connect_to_file_query().await?;
    let file_versions = Rpc::list_versions(&mut file_query, file_id).await?;

    let mut table = StdTable::new(
        vec!["id", "size, B", "hash", "name", "state", "tags"],
        format,
    );

    for v in file_versions.into_iter() {
        let mut row = vec![
            v.id.to_string(),
            v.size.to_string(),
            v.hash,
            v.name,
            if v.is_deleted {
                "deleted".to_string()
            } else {
                "active".to_string()
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
