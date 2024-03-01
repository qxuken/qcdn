use color_eyre::Result;

use crate::{cli::Cli, rpc::Rpc, utils::std_table::StdTable};

#[tracing::instrument(skip_all)]
pub async fn tag_version(cli: &Cli, file_version_id: i64, tag: String) -> Result<()> {
    let rpc: Rpc = cli.into();

    let mut file_updates = rpc.connect_to_file_updates().await?;
    Rpc::tag_version(&mut file_updates, file_version_id, tag).await?;

    let mut file_query = rpc.connect_to_file_query().await?;
    let v = Rpc::get_version(&mut file_query, file_version_id).await?;
    tracing::trace!("{v:?}");

    let mut table = StdTable::new(
        vec!["id", "size, B", "name", "state", "tags"],
        Default::default(),
    );

    let mut row = vec![
        v.id.to_string(),
        v.size.to_string(),
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

    println!("{table}");
    println!("Ok");

    Ok(())
}
