use chrono::{NaiveDateTime, Utc};
use diesel::{delete, prelude::*, update};
use serde::{Deserialize, Serialize};

use crate::{DatabaseConnection, DatabaseError, FileVersionTag};

use super::File;

pub use file_version_path_parts::FileVersionPathParts;
pub use file_version_state::FileVersionState;
pub use new_file_version::NewFileVersion;

mod file_version_path_parts;
mod file_version_state;
mod new_file_version;

#[derive(
    Debug,
    Deserialize,
    Serialize,
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Associations,
    PartialEq,
    Eq,
)]
#[diesel(belongs_to(File))]
#[diesel(table_name = crate::schema::file_version)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct FileVersion {
    pub id: i64,
    pub file_id: i64,
    pub size: i64,
    pub version: String,
    pub state: FileVersionState,
    pub created_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl FileVersion {
    pub fn find_by_file_id(
        connection: &mut DatabaseConnection,
        file_id: i64,
    ) -> Result<Vec<Self>, DatabaseError> {
        use crate::schema::file_version::dsl;

        dsl::file_version
            .filter(dsl::file_id.eq(file_id))
            .select(Self::as_select())
            .get_results(connection)
            .map_err(DatabaseError::from)
    }

    pub fn find_by_id(connection: &mut DatabaseConnection, id: i64) -> Result<Self, DatabaseError> {
        use crate::schema::file_version::dsl;

        dsl::file_version
            .find(id)
            .select(Self::as_select())
            .first(connection)
            .map_err(DatabaseError::from)
    }

    pub fn find_by_version_optional(
        connection: &mut DatabaseConnection,
        file_id: &i64,
        version: &str,
    ) -> Result<Option<Self>, DatabaseError> {
        use crate::schema::file_version::dsl;

        dsl::file_version
            .filter(dsl::file_id.eq(file_id).and(dsl::version.eq(version)))
            .select(Self::as_select())
            .first(connection)
            .optional()
            .map_err(DatabaseError::from)
    }
}

impl FileVersion {
    pub fn path(
        &self,
        connection: &mut DatabaseConnection,
    ) -> Result<FileVersionPathParts, DatabaseError> {
        use crate::schema::dir::dsl as d_dsl;
        use crate::schema::file::dsl as f_dsl;

        let (dir, file) = f_dsl::file
            .find(self.file_id)
            .inner_join(d_dsl::dir)
            .select((d_dsl::name, f_dsl::name))
            .get_result::<(String, String)>(connection)?;

        Ok(FileVersionPathParts {
            dir,
            file,
            version: self.version.to_owned(),
        })
    }

    pub fn update_state(
        &mut self,
        connection: &mut DatabaseConnection,
        state: FileVersionState,
    ) -> Result<(), DatabaseError> {
        use crate::schema::file_version::dsl;

        update(&*self)
            .set(dsl::state.eq(state))
            .execute(connection)?;

        self.state = state;

        Ok(())
    }

    pub fn delete(&mut self, connection: &mut DatabaseConnection) -> Result<(), DatabaseError> {
        use crate::schema::file_version::dsl;

        let deleted_at = Utc::now().naive_utc();

        update(&*self)
            .set(dsl::deleted_at.eq(deleted_at))
            .execute(connection)?;

        self.deleted_at = Some(deleted_at);

        Ok(())
    }

    pub fn unsafe_delete(&self, connection: &mut DatabaseConnection) -> Result<(), DatabaseError> {
        if self.state == FileVersionState::Ready {
            return DatabaseError::PreconditionError(
                "Versions with ready state cannot be deleted".to_string(),
            )
            .as_err();
        }

        connection.transaction::<_, DatabaseError, _>(|tx| {
            delete(FileVersionTag::belonging_to(self)).execute(tx)?;

            delete(self).execute(tx)?;

            Ok(())
        })
    }
}
