use chrono::NaiveDateTime;
use diesel::{delete, prelude::*};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{DatabaseConnection, DatabaseError, File};

pub use dir_upsert::DirUpsert;

mod dir_upsert;

#[derive(Debug, Deserialize, Serialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::dir)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Dir {
    pub id: i64,
    pub name: String,
    pub created_at: NaiveDateTime,
}

impl Dir {
    #[instrument(skip(connection))]
    pub async fn get_all(connection: &mut DatabaseConnection) -> Result<Vec<Self>, DatabaseError> {
        use crate::schema::dir::dsl;

        dsl::dir
            .select(Self::as_select())
            .get_results(connection)
            .map_err(DatabaseError::from)
    }

    #[instrument(skip(connection))]
    pub async fn find_by_id(
        connection: &mut DatabaseConnection,
        id: i64,
    ) -> Result<Self, DatabaseError> {
        use crate::schema::dir::dsl;

        dsl::dir
            .find(id)
            .select(Self::as_select())
            .first(connection)
            .map_err(DatabaseError::from)
    }

    #[instrument(skip(connection))]
    pub fn find_by_name_optional(
        connection: &mut DatabaseConnection,
        name: &str,
    ) -> Result<Option<Self>, DatabaseError> {
        use crate::schema::dir::dsl;

        dsl::dir
            .filter(dsl::name.eq(name))
            .select(Self::as_select())
            .first(connection)
            .optional()
            .map_err(DatabaseError::from)
    }
}

impl Dir {
    #[instrument(skip(connection))]
    pub fn delete_if_no_files_exists(
        &self,
        connection: &mut DatabaseConnection,
    ) -> Result<(), DatabaseError> {
        let files_count = File::belonging_to(self)
            .count()
            .get_result::<i64>(connection)?;

        if files_count != 0 {
            return Ok(());
        }

        delete(self).execute(connection)?;
        Ok(())
    }
}
