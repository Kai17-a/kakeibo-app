use crate::{
    handler::import::{self as handler, AppState},
    repository::import::ImportRepository,
    service::import::ImportService,
};
use axum::{Router, routing::post};
use sqlx::SqlitePool;
pub fn create(pool: SqlitePool) -> Router {
    let state = AppState {
        import: ImportService::new(ImportRepository::new(pool)),
    };
    Router::new()
        .route("/api/import/expenses", post(handler::expenses))
        .route("/api/import/incomes", post(handler::incomes))
        .with_state(state)
}
