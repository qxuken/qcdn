use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};
use sqlx::SqlitePool;

use crate::app_state::AppState;

pub type DatabasePoolConnection = sqlx::pool::PoolConnection<sqlx::Sqlite>;

pub struct DatabaseConnection(pub DatabasePoolConnection);

impl FromRef<AppState> for SqlitePool {
    fn from_ref(app_state: &AppState) -> SqlitePool {
        app_state.db.inner()
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for DatabaseConnection
where
    SqlitePool: FromRef<AppState>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let pool = SqlitePool::from_ref(state);

        let conn = pool
            .acquire()
            .await
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

        Ok(Self(conn))
    }
}
