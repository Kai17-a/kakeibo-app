use crate::{
    handler::expenses::{self as handler, AppState},
    repository::expenses::ExpenseRepository,
    service::expenses::ExpenseService,
};
use axum::{Router, routing::get};
use sqlx::SqlitePool;
pub fn create(pool: SqlitePool) -> Router {
    let state = AppState {
        expenses: ExpenseService::new(ExpenseRepository::new(pool)),
    };
    Router::new()
        .route("/api/expenses", get(handler::list).post(handler::create))
        .route(
            "/api/expenses/{id}",
            get(handler::get)
                .put(handler::update)
                .delete(handler::delete),
        )
        .with_state(state)
}
