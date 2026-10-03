use crate::{
    handler::export::{self as handler, AppState},
    repository::export::ExportRepository,
    service::export::ExportService,
};
use axum::{Router, routing::get};
use sqlx::SqlitePool;
pub fn create(pool: SqlitePool) -> Router {
    let state = AppState {
        export: ExportService::new(ExportRepository::new(pool)),
    };
    Router::new()
        .route("/api/export/expenses", get(handler::expenses))
        .route("/api/export/incomes", get(handler::incomes))
        .route("/api/export/transfers", get(handler::transfers))
        .with_state(state)
}
