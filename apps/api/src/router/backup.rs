use crate::{
    handler::backup::{self as handler, AppState},
    service::backup::{BackupService, RestoreConfig},
};
use axum::{
    Router,
    routing::{get, post},
};
use sqlx::SqlitePool;

pub fn create(pool: SqlitePool, restore_config: RestoreConfig) -> Router {
    create_with_service(BackupService::new(pool, restore_config))
}

pub fn create_with_service(backup: BackupService) -> Router {
    let state = AppState { backup };
    Router::new()
        .route("/api/backup", get(handler::get))
        .route("/api/backup/restore", post(handler::restore))
        .with_state(state)
}
