use crate::{
    handler::backup::{self as handler, AppState},
    service::backup::BackupService,
};
use axum::{Router, routing::get};
use sqlx::SqlitePool;

pub fn create(pool: SqlitePool) -> Router {
    let state = AppState {
        backup: BackupService::new(pool),
    };
    Router::new()
        .route("/api/backup", get(handler::get))
        .with_state(state)
}
