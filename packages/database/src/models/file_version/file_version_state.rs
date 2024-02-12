use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::SmallInt};
use diesel_enum::DbEnum;
use serde::{Deserialize, Serialize};

use crate::DatabaseError;

#[derive(
    Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, AsExpression, FromSqlRow, DbEnum,
)]
#[diesel(sql_type = SmallInt)]
#[diesel_enum(error_fn = DatabaseError::NotFound)]
#[diesel_enum(error_type = DatabaseError)]
pub enum FileVersionState {
    Created,
    Downloading,
    Ready,
}
