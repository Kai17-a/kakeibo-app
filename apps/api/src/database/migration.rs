use crate::utils::error::{AppError, AppResult};
use sqlx::{
    SqlitePool,
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use tracing::{info, instrument};

pub(crate) static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[instrument(name = "database.migrate", skip_all)]
pub async fn connect() -> AppResult<SqlitePool> {
    connect_path("kakeibo.db").await
}

pub async fn connect_path(path: impl AsRef<std::path::Path>) -> AppResult<SqlitePool> {
    info!("Opening database");

    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|error| AppError::context("Failed to open the database", error))?;

    MIGRATOR
        .run(&pool)
        .await
        .map_err(|error| AppError::context("Failed to apply migrations", error))?;

    info!("Database migrations applied");
    Ok(pool)
}
