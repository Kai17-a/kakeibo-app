use crate::{
    handler::recurring_incomes::{self as handler, AppState},
    repository::recurring_incomes::RecurringIncomeRepository,
    service::recurring_incomes::RecurringIncomeService,
};
use axum::{Router, routing::get};
use sqlx::SqlitePool;

pub fn create(pool: SqlitePool) -> Router {
    let state = AppState {
        recurring_incomes: RecurringIncomeService::new(RecurringIncomeRepository::new(pool)),
    };
    Router::new()
        .route(
            "/api/recurring-incomes",
            get(handler::list).post(handler::create),
        )
        .route(
            "/api/recurring-incomes/{id}",
            get(handler::get)
                .put(handler::update)
                .delete(handler::delete),
        )
        .with_state(state)
}
