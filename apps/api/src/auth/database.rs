//! The SQLite database that stores accounts and sessions.

use crate::utils::error::{AppError, AppResult};
use sqlx::{
    SqlitePool,
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::{path::Path, time::Duration};
use tracing::info;

static AUTH_MIGRATOR: Migrator = sqlx::migrate!("./migrations_auth");

pub async fn connect(path: impl AsRef<Path>) -> AppResult<SqlitePool> {
    info!("Opening authentication database");
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5));
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|error| AppError::context("Failed to open the authentication database", error))?;
    migrate(&pool).await?;
    Ok(pool)
}

pub async fn migrate(pool: &SqlitePool) -> AppResult<()> {
    AUTH_MIGRATOR.run(pool).await.map_err(|error| {
        AppError::context("Failed to apply authentication database migrations", error)
    })
}
