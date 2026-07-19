use crate::utils::error::{AppError, AppResult};
use sqlx::{
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn migrate() -> AppResult<()> {
    let options = SqliteConnectOptions::new()
        .filename("kakeibo.db")
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

    Ok(())
}
