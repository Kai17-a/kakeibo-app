use axum::{Router, routing::get};
use sqlx::SqlitePool;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    handler::incomes::{self as handler, AppState},
    model::incomes::{Income, IncomeInput, IncomeSortBy, SortOrder},
    repository::incomes::IncomeRepository,
    service::incomes::IncomeService,
};

#[derive(OpenApi)]
#[openapi(
    paths(handler::list, handler::get, handler::create, handler::update, handler::delete),
    components(schemas(Income, IncomeInput, IncomeSortBy, SortOrder)),
    tags((name = "incomes", description = "Income CRUD API"))
)]
struct ApiDoc;

pub fn create(pool: SqlitePool) -> Router {
    let state = AppState {
        incomes: IncomeService::new(IncomeRepository::new(pool)),
    };
    Router::new()
        .route("/api/incomes", get(handler::list).post(handler::create))
        .route(
            "/api/incomes/{id}",
            get(handler::get)
                .put(handler::update)
                .delete(handler::delete),
        )
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}
