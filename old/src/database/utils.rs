use chrono::{DateTime, LocalResult, TimeZone, Utc};
use sqlx::{sqlite::SqliteRow, Row};

pub fn parse_uuid(row: &SqliteRow, field_name: &str) -> Result<uuid::Uuid, sqlx::Error> {
    let value: String = row.try_get(field_name)?;
    uuid::Uuid::parse_str(&value).map_err(|e| sqlx::Error::ColumnDecode {
        index: field_name.to_string(),
        source: e.into(),
    })
}

pub fn parse_timestamp(row: &SqliteRow, field_name: &str) -> Result<DateTime<Utc>, sqlx::Error> {
    let created_at: i64 = row.try_get(field_name)?;
    match Utc.timestamp_opt(created_at, 0) {
        LocalResult::Single(res) => Ok(res),
        LocalResult::Ambiguous(min, _max) => Ok(min),
        LocalResult::None => {
            let err = anyhow::anyhow!("{field_name} decode error");
            Err(sqlx::Error::ColumnDecode {
                index: field_name.to_string(),
                source: err.into(),
            })
        }
    }
}
