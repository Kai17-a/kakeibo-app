use crate::{
    handler::expenses::{self as handler, AppState},
    repository::{expenses::ExpenseRepository, webhook_urls::WebhookUrlRepository},
    service::{
        exchange_rate::ExchangeRateService, expenses::ExpenseService,
        webhook_urls::WebhookUrlService,
    },
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
        expenses: ExpenseService::new(
            ExpenseRepository::new(pool.clone()),
            ExchangeRateService::with_provider(pool.clone(), provider),
        ),
        webhook_urls: WebhookUrlService::new(WebhookUrlRepository::new(pool)),
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
