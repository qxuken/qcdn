use chrono::{NaiveDateTime, Utc};
use diesel::{prelude::*, update};
use tracing::instrument;
use crate::{DatabaseConnection, DatabaseError};

use super::FileVersion;
pub use file_version_tag_upsert::FileVersionTagUpsert;

mod file_version_tag_upsert;

#[derive(Queryable, Selectable, Identifiable, Associations, AsChangeset, PartialEq, Eq, Debug)]
#[diesel(belongs_to(FileVersion))]
#[diesel(table_name = crate::schema::file_version_tag)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct FileVersionTag {
    pub id: i64,
    pub file_version_id: i64,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub activated_at: NaiveDateTime,
}

impl FileVersionTag {
    #[instrument(skip(connection))]
    pub fn find_by_file_version(connection: &mut DatabaseConnection, file_version_id: &i64) -> Result<Vec<Self>, DatabaseError> {
        use crate::schema::file_version_tag::dsl;

        dsl::file_version_tag
            .filter(dsl::file_version_id.eq(file_version_id))
            .select(Self::as_select())
            .get_results(connection)
            .map_err(DatabaseError::from)
    }

    #[instrument(skip(connection))]
    pub fn find_by_name_optional(connection: &mut DatabaseConnection, file_version_id: &i64, name: &str) -> Result<Option<Self>, DatabaseError> {
        use crate::schema::file_version_tag::dsl;

        dsl::file_version_tag
            .filter(dsl::file_version_id.eq(file_version_id))
            .filter(dsl::name.eq(name))
            .select(Self::as_select())
            .first(connection)
            .optional()
            .map_err(DatabaseError::from)
    }
}


impl FileVersionTag {
    #[instrument(skip(connection))]
    pub fn move_to_version(&mut self, connection: &mut DatabaseConnection, file_version_id: &i64) -> Result<(), DatabaseError> {
        use crate::schema::file_version_tag::dsl;

        let activated_at = update(&*self)
            .set((dsl::file_version_id.eq(file_version_id), dsl::activated_at.eq(Utc::now().naive_utc())))
            .returning(dsl::activated_at)
            .get_result(connection)?;

        self.activated_at = activated_at;
        self.file_version_id = *file_version_id;
        Ok(())
    }
}
