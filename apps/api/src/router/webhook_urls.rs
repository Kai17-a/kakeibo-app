use crate::{
    handler::webhook_urls::{self as handler, AppState},
    repository::webhook_urls::WebhookUrlRepository,
    service::webhook_urls::WebhookUrlService,
};
use axum::{Router, routing::get};
use sqlx::SqlitePool;
pub fn create(pool: SqlitePool) -> Router {
    let state = AppState {
        webhook_urls: WebhookUrlService::new(WebhookUrlRepository::new(pool)),
    };
    Router::new()
        .route(
            "/api/webhook-urls",
            get(handler::list).post(handler::create),
        )
        .route(
            "/api/webhook-urls/{id}",
            get(handler::get)
                .put(handler::update)
                .delete(handler::delete),
        )
        .with_state(state)
}
