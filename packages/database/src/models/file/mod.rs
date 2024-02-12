use chrono::NaiveDateTime;
use diesel::{delete, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{DatabaseConnection, DatabaseError};

use super::{Dir, FileVersion};

pub use file_type::FileType;
pub use file_upsert::FileUpsert;

mod file_type;
mod file_upsert;

#[derive(
    Debug, Deserialize, Serialize, Queryable, Selectable, Identifiable, Associations, PartialEq, Eq,
)]
#[diesel(belongs_to(Dir))]
#[diesel(table_name = crate::schema::file)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct File {
    pub id: i64,
    pub dir_id: i64,
    pub name: String,
    pub file_type: FileType,
    pub created_at: NaiveDateTime,
}

impl File {
    pub fn get_all(
        connection: &mut DatabaseConnection,
    ) -> Result<Vec<Self>, DatabaseError> {
        use crate::schema::file::dsl;

        dsl::file
            .select(Self::as_select())
            .get_results(connection)
            .map_err(DatabaseError::from)
    }

    pub fn find_all_by_dir(
        connection: &mut DatabaseConnection,
        dir: &Dir,
        offset: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<Self>, DatabaseError> {
        Self::belonging_to(dir)
            .offset(offset.unwrap_or_default())
            .limit(limit.unwrap_or(10))
            .get_results(connection)
            .map_err(DatabaseError::from)
    }

    pub async fn find_by_id(
        connection: &mut DatabaseConnection,
        id: i64,
    ) -> Result<Self, DatabaseError> {
        use crate::schema::file::dsl;

        dsl::file
            .find(id)
            .select(Self::as_select())
            .first(connection)
            .map_err(DatabaseError::from)
    }

    pub fn find_by_name_optional(
        connection: &mut DatabaseConnection,
        dir_id: &i64,
        name: &str,
    ) -> Result<Option<Self>, DatabaseError> {
        use crate::schema::file::dsl;

        dsl::file
            .filter(dsl::dir_id.eq(dir_id).and(dsl::name.eq(name)))
            .first(connection)
            .optional()
            .map_err(DatabaseError::from)
    }
}

impl File {
    pub fn delete_if_no_versions_exists(
        &self,
        connection: &mut DatabaseConnection,
    ) -> Result<(), DatabaseError> {
        let files_count = FileVersion::belonging_to(self)
            .count()
            .get_result::<i64>(connection)?;

        if files_count != 0 {
            return Ok(());
        }

        delete(self).execute(connection)?;
        Ok(())
    }
}
