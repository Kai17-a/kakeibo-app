use crate::{
    handler::import::{self as handler, AppState},
    repository::import::ImportRepository,
    service::import::ImportService,
};
use axum::{
    Router,
    routing::{get, post},
};
use sqlx::SqlitePool;
pub fn create(pool: SqlitePool) -> Router {
    let state = AppState {
        import: ImportService::new(ImportRepository::new(pool)),
    };
    Router::new()
        .route("/api/import/expenses", post(handler::expenses))
        .route("/api/import/expenses/sample", get(handler::expense_sample))
        .route("/api/import/incomes", post(handler::incomes))
        .route("/api/import/incomes/sample", get(handler::income_sample))
        .with_state(state)
}
