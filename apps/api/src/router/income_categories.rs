use axum::{Router, routing::get};
use sqlx::SqlitePool;

use crate::{
    handler::income_categories::{self as handler, AppState},
    repository::income_categories::IncomeCategoryRepository,
    service::income_categories::IncomeCategoryService,
};

pub fn create(pool: SqlitePool) -> Router {
    let state = AppState {
        income_categories: IncomeCategoryService::new(IncomeCategoryRepository::new(pool)),
    };
    Router::new()
        .route(
            "/api/income-categories/order",
            axum::routing::put(handler::reorder),
        )
        .route(
            "/api/income-categories",
            get(handler::list).post(handler::create),
        )
        .route(
            "/api/income-categories/{id}",
            get(handler::get)
                .put(handler::update)
                .delete(handler::delete),
        )
        .with_state(state)
}
