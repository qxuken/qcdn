use chrono::{NaiveDateTime, Utc};
use diesel::{insert_into, prelude::*};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{DatabaseConnection, DatabaseError};

use super::{FileVersion, FileVersionState};

#[derive(Debug, Deserialize, Serialize, Insertable)]
#[diesel(table_name = crate::schema::file_version)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewFileVersion {
    pub file_id: i64,
    pub size: i64,
    pub version: String,
    pub state: FileVersionState,
    pub created_at: Option<NaiveDateTime>,
}

impl NewFileVersion {
    #[instrument(skip(connection))]
    pub fn create(
        mut self,
        connection: &mut DatabaseConnection,
    ) -> Result<FileVersion, DatabaseError> {
        use crate::schema::file_version::dsl;

        if FileVersion::find_by_version_optional(connection, &self.file_id, &self.version)?
            .is_some()
        {
            return DatabaseError::PreconditionError("Version is already exists".to_string()).err();
        }

        if self.created_at.is_none() {
            self.created_at = Some(Utc::now().naive_utc())
        }

        insert_into(dsl::file_version)
            .values(&self)
            .returning(FileVersion::as_returning())
            .get_result(connection)
            .map_err(DatabaseError::from)
    }
}
