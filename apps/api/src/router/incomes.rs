use axum::{Router, routing::get};
use sqlx::SqlitePool;

use crate::{
    handler::incomes::{self as handler, AppState},
    repository::{incomes::IncomeRepository, webhook_urls::WebhookUrlRepository},
    service::{incomes::IncomeService, webhook_urls::WebhookUrlService},
};

pub fn create(pool: SqlitePool) -> Router {
    let state = AppState {
        incomes: IncomeService::new(IncomeRepository::new(pool.clone())),
        webhook_urls: WebhookUrlService::new(WebhookUrlRepository::new(pool)),
    };
    Router::new()
        .route("/api/incomes", get(handler::list).post(handler::create))
        .route(
            "/api/incomes/{id}",
            get(handler::get)
                .put(handler::update)
                .delete(handler::delete),
        )
        .with_state(state)
}
