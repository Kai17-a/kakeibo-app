use crate::{
    handler::recurring_transfers::{self as handler, AppState},
    repository::recurring_transfers::RecurringTransferRepository,
    service::recurring_transfers::RecurringTransferService,
};
use axum::{Router, routing::get};
use sqlx::SqlitePool;

pub fn create(pool: SqlitePool) -> Router {
    let state = AppState {
        recurring_transfers: RecurringTransferService::new(RecurringTransferRepository::new(pool)),
    };
    Router::new()
        .route(
            "/api/recurring-transfers",
            get(handler::list).post(handler::create),
        )
        .route(
            "/api/recurring-transfers/{id}",
            get(handler::get)
                .put(handler::update)
                .delete(handler::delete),
        )
        .with_state(state)
}
