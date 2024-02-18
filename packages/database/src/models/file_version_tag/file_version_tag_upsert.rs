use chrono::{NaiveDateTime, Utc};
use diesel::{insert_into, prelude::*};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{DatabaseConnection, DatabaseError};

use super::FileVersionTag;

#[derive(Debug, Deserialize, Serialize, Insertable)]
#[diesel(table_name = crate::schema::file_version_tag)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct FileVersionTagUpsert {
    pub file_version_id: i64,
    pub name: String,
    pub created_at: Option<NaiveDateTime>,
}

impl FileVersionTagUpsert {
    #[instrument(skip(connection))]
    fn create(self, connection: &mut DatabaseConnection) -> Result<FileVersionTag, DatabaseError> {
        use crate::schema::file_version_tag::dsl;
        let created_at = self.created_at.unwrap_or_else(|| Utc::now().naive_utc());

        insert_into(dsl::file_version_tag)
            .values((
                &self,
                dsl::created_at.eq(created_at),
                dsl::activated_at.eq(created_at),
            ))
            .returning(FileVersionTag::as_returning())
            .get_result(connection)
            .map_err(DatabaseError::from)
    }

    #[instrument(skip(connection))]
    pub fn create_or_move(
        self,
        connection: &mut DatabaseConnection,
    ) -> Result<FileVersionTag, DatabaseError> {
        let item = match FileVersionTag::find_by_name_optional(
            connection,
            &self.file_version_id,
            &self.name,
        )? {
            Some(mut tag) => {
                tag.move_to_version(connection, &self.file_version_id)?;
                tag
            }
            None => self.create(connection)?,
        };

        Ok(item)
    }
}
