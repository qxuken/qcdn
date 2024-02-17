use chrono::{NaiveDateTime, Utc};
use diesel::{insert_into, prelude::*};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{DatabaseConnection, DatabaseError};

use super::{File, FileType};

#[derive(Debug, Deserialize, Serialize, Insertable)]
#[diesel(table_name = crate::schema::file)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct FileUpsert {
    pub dir_id: i64,
    pub name: String,
    pub file_type: FileType,
    pub created_at: Option<NaiveDateTime>,
}

impl FileUpsert {
    #[instrument(skip(connection))]
    fn create(mut self, connection: &mut DatabaseConnection) -> Result<File, DatabaseError> {
        use crate::schema::file::dsl;

        if self.created_at.is_none() {
            self.created_at = Some(Utc::now().naive_utc())
        }

        insert_into(dsl::file)
            .values(&self)
            .returning(File::as_returning())
            .get_result(connection)
            .map_err(DatabaseError::from)
    }

    #[instrument(skip(connection))]
    pub fn find_by_name_or_create(
        self,
        connection: &mut DatabaseConnection,
    ) -> Result<File, DatabaseError> {
        File::find_by_name_optional(connection, &self.dir_id, &self.name)
            .and_then(|r| r.map(Ok).unwrap_or_else(|| self.create(connection)))
    }
}
