use crate::{
    handler::recurring_expenses::{self as handler, AppState},
    repository::recurring_expenses::RecurringExpenseRepository,
    service::{exchange_rate::ExchangeRateService, recurring_expenses::RecurringExpenseService},
};
use axum::{Router, routing::get};
use sqlx::SqlitePool;
use std::sync::Arc;
pub fn create(pool: SqlitePool) -> Router {
    let provider = Arc::new(
        crate::service::exchange_rate::FrankfurterProvider::new()
            .expect("exchange-rate client must initialize"),
    );
    create_with_exchange_rate_provider(pool, provider)
}

pub fn create_with_exchange_rate_provider(
    pool: SqlitePool,
    provider: Arc<dyn crate::service::exchange_rate::ExchangeRateProvider>,
) -> Router {
    let state = AppState {
        recurring_expenses: RecurringExpenseService::new(
            RecurringExpenseRepository::new(pool.clone()),
            ExchangeRateService::with_provider(pool, provider),
        ),
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
        .route(
            "/api/recurring-expenses/{id}/exchange-rate",
            get(handler::exchange_rate_preview),
        )
        .with_state(state)
}
