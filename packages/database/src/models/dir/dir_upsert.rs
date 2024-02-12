use chrono::{NaiveDateTime, Utc};
use diesel::{insert_into, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{DatabaseConnection, DatabaseError};

use super::Dir;

#[derive(Debug, Deserialize, Serialize, Insertable)]
#[diesel(table_name = crate::schema::dir)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DirUpsert {
    pub name: String,
    pub created_at: Option<NaiveDateTime>,
}

impl DirUpsert {
    #[instrument(skip(connection))]
    pub fn create(mut self, connection: &mut DatabaseConnection) -> Result<Dir, DatabaseError> {
        use crate::schema::dir::dsl;

        if self.created_at.is_none() {
            self.created_at = Some(Utc::now().naive_utc())
        }

        insert_into(dsl::dir)
            .values(&self)
            .returning(Dir::as_returning())
            .get_result(connection)
            .map_err(DatabaseError::from)
    }

    pub async fn find_by_name_or_create(
        self,
        connection: &mut DatabaseConnection,
    ) -> Result<Dir, DatabaseError> {
        Dir::find_by_name_optional(connection, &self.name)
            .and_then(|r| r.map(Ok).unwrap_or_else(|| self.create(connection)))
    }
}
