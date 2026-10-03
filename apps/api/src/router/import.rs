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
        .route(
            "/api/import/expenses/preview",
            post(handler::preview_expenses),
        )
        .route("/api/import/expenses/sample", get(handler::expense_sample))
        .route("/api/import/incomes", post(handler::incomes))
        .route(
            "/api/import/incomes/preview",
            post(handler::preview_incomes),
        )
        .route("/api/import/incomes/sample", get(handler::income_sample))
        .route(
            "/api/import/recurring-expenses",
            post(handler::recurring_expenses),
        )
        .route(
            "/api/import/recurring-expenses/preview",
            post(handler::preview_recurring_expenses),
        )
        .route(
            "/api/import/recurring-expenses/sample",
            get(handler::recurring_expense_sample),
        )
        .route(
            "/api/import/variable-expenses",
            post(handler::variable_expenses),
        )
        .route(
            "/api/import/variable-expenses/preview",
            post(handler::preview_variable_expenses),
        )
        .route(
            "/api/import/variable-expenses/sample",
            get(handler::variable_expense_sample),
        )
        .route("/api/import/transfers", post(handler::transfers))
        .route(
            "/api/import/transfers/preview",
            post(handler::preview_transfers),
        )
        .route(
            "/api/import/transfers/sample",
            get(handler::transfer_sample),
        )
        .with_state(state)
}
