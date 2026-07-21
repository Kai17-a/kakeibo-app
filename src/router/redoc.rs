use axum::Router;
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};

use crate::{
    handler::{income_categories, incomes},
    model::income_categories::{
        IncomeCategory, IncomeCategoryListResponse, IncomeCategoryPagination, IncomeCategorySortBy,
        IncomeCategorySortOrder, IncomeCategoryUpsertRequest,
    },
    model::incomes::{
        Income, IncomeListResponse, IncomeSortBy, IncomeUpsertRequest, Pagination, SortOrder,
    },
};

#[derive(OpenApi)]
#[openapi(
    paths(
        income_categories::list,
        income_categories::get,
        income_categories::create,
        income_categories::update,
        income_categories::delete,
        incomes::list,
        incomes::get,
        incomes::create,
        incomes::update,
        incomes::delete
    ),
    components(schemas(
        IncomeCategory,
        IncomeCategoryUpsertRequest,
        IncomeCategoryListResponse,
        IncomeCategoryPagination,
        IncomeCategorySortBy,
        IncomeCategorySortOrder,
        Income,
        IncomeUpsertRequest,
        IncomeListResponse,
        Pagination,
        IncomeSortBy,
        SortOrder
    )),
    tags(
        (name = "income categories", description = "Income category CRUD API"),
        (name = "incomes", description = "Income CRUD API")
    )
)]
struct ApiDoc;

pub fn create() -> Router {
    Router::new().merge(Redoc::with_url("/docs", ApiDoc::openapi()))
}
