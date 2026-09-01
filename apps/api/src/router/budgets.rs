use crate::{
    handler::budgets::{self as handler, AppState},
    repository::budgets::BudgetRepository,
    service::budgets::BudgetService,
};
use axum::{Router, routing::get};
use sqlx::SqlitePool;
pub fn create(pool: SqlitePool) -> Router {
    Router::new()
        .route("/api/budgets", get(handler::list).post(handler::create))
        .route(
            "/api/budgets/{id}",
            get(handler::get)
                .put(handler::update)
                .delete(handler::delete),
        )
        .with_state(AppState {
            budgets: BudgetService::new(BudgetRepository::new(pool)),
        })
}
