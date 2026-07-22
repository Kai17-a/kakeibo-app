use crate::{
    handler::payment_methods::{self as handler, AppState},
    repository::payment_methods::PaymentMethodRepository,
    service::payment_methods::PaymentMethodService,
};
use axum::{Router, routing::get};
use sqlx::SqlitePool;
pub fn create(pool: SqlitePool) -> Router {
    let state = AppState {
        payment_methods: PaymentMethodService::new(PaymentMethodRepository::new(pool)),
    };
    Router::new()
        .route(
            "/api/payment-methods",
            get(handler::list).post(handler::create),
        )
        .route(
            "/api/payment-methods/{id}",
            get(handler::get)
                .put(handler::update)
                .delete(handler::delete),
        )
        .with_state(state)
}
