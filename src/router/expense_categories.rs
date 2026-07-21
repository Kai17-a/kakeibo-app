use crate::{
    handler::expense_categories::{self as handler, AppState},
    repository::expense_categories::ExpenseCategoryRepository,
    service::expense_categories::ExpenseCategoryService,
};
use axum::{Router, routing::get};
use sqlx::SqlitePool;
pub fn create(pool: SqlitePool) -> Router {
    let state = AppState {
        expense_categories: ExpenseCategoryService::new(ExpenseCategoryRepository::new(pool)),
    };
    Router::new()
        .route(
            "/api/expense-categories",
            get(handler::list).post(handler::create),
        )
        .route(
            "/api/expense-categories/{id}",
            get(handler::get)
                .put(handler::update)
                .delete(handler::delete),
        )
        .with_state(state)
}
