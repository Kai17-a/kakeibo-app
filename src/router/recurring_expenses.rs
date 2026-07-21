use crate::{
    handler::recurring_expenses::{self as handler, AppState},
    repository::recurring_expenses::RecurringExpenseRepository,
    service::recurring_expenses::RecurringExpenseService,
};
use axum::{Router, routing::get};
use sqlx::SqlitePool;
pub fn create(pool: SqlitePool) -> Router {
    let state = AppState {
        recurring_expenses: RecurringExpenseService::new(RecurringExpenseRepository::new(pool)),
    };
    Router::new()
        .route(
            "/api/recurring-expenses",
            get(handler::list).post(handler::create),
        )
        .route(
            "/api/recurring-expenses/{id}",
            get(handler::get)
                .put(handler::update)
                .delete(handler::delete),
        )
        .with_state(state)
}
