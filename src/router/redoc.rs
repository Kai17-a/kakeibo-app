use axum::Router;
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};

use crate::{
    handler::incomes,
    model::incomes::{
        Income, IncomeListResponse, IncomeSortBy, IncomeUpsertRequest, Pagination, SortOrder,
    },
};

#[derive(OpenApi)]
#[openapi(
    paths(
        incomes::list,
        incomes::get,
        incomes::create,
        incomes::update,
        incomes::delete
    ),
    components(schemas(
        Income,
        IncomeUpsertRequest,
        IncomeListResponse,
        Pagination,
        IncomeSortBy,
        SortOrder
    )),
    tags((name = "incomes", description = "Income CRUD API"))
)]
struct ApiDoc;

pub fn create() -> Router {
    Router::new().merge(Redoc::with_url("/docs", ApiDoc::openapi()))
}
