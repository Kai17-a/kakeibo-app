use crate::{
    handler::transfers::{self as handler, AppState},
    repository::transfers::TransferRepository,
    service::transfers::TransferService,
};
use axum::{Router, routing::get};
use sqlx::SqlitePool;
pub fn create(pool: SqlitePool) -> Router {
    let state = AppState {
        transfers: TransferService::new(TransferRepository::new(pool)),
    };
    Router::new()
        .route("/api/transfers", get(handler::list).post(handler::create))
        .route(
            "/api/transfers/{id}",
            get(handler::get)
                .put(handler::update)
                .delete(handler::delete),
        )
        .with_state(state)
}
