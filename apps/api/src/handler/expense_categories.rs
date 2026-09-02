use crate::{
    model::expense_categories::{
        ExpenseCategory, ExpenseCategoryListResponse, ExpenseCategoryQuery,
        ExpenseCategoryReorderRequest, ExpenseCategoryUpsertRequest,
    },
    service::expense_categories::ExpenseCategoryService,
    utils::error::{AppError, AppResult},
};
use axum::{
    Json,
    extract::{Path, Query, State, rejection::QueryRejection},
    http::StatusCode,
};
#[derive(Clone)]
pub struct AppState {
    pub expense_categories: ExpenseCategoryService,
}
#[utoipa::path(get,path="/api/expense-categories",params(ExpenseCategoryQuery),responses((status=200,body=ExpenseCategoryListResponse),(status=400)))]
pub async fn list(
    State(s): State<AppState>,
    q: Result<Query<ExpenseCategoryQuery>, QueryRejection>,
) -> AppResult<Json<ExpenseCategoryListResponse>> {
    let Query(q) =
        q.map_err(|e| AppError::bad_request(&format!("Invalid query parameters: {e}")))?;
    Ok(Json(s.expense_categories.list(&q).await?))
}
#[utoipa::path(get,path="/api/expense-categories/{id}",params(("id"=String,Path)),responses((status=200,body=ExpenseCategory),(status=404)))]
pub async fn get(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<ExpenseCategory>> {
    Ok(Json(s.expense_categories.get(&id).await?))
}
#[utoipa::path(post,path="/api/expense-categories",request_body=ExpenseCategoryUpsertRequest,responses((status=201,body=ExpenseCategory),(status=400)))]
pub async fn create(
    State(s): State<AppState>,
    Json(v): Json<ExpenseCategoryUpsertRequest>,
) -> AppResult<(StatusCode, Json<ExpenseCategory>)> {
    Ok((
        StatusCode::CREATED,
        Json(s.expense_categories.create(&v).await?),
    ))
}
#[utoipa::path(put,path="/api/expense-categories/{id}",params(("id"=String,Path)),request_body=ExpenseCategoryUpsertRequest,responses((status=200,body=ExpenseCategory),(status=400),(status=404)))]
pub async fn update(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(v): Json<ExpenseCategoryUpsertRequest>,
) -> AppResult<Json<ExpenseCategory>> {
    Ok(Json(s.expense_categories.update(&id, &v).await?))
}
#[utoipa::path(delete,path="/api/expense-categories/{id}",params(("id"=String,Path)),responses((status=204),(status=404)))]
pub async fn delete(State(s): State<AppState>, Path(id): Path<String>) -> AppResult<StatusCode> {
    s.expense_categories.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
#[utoipa::path(put,path="/api/expense-categories/order",request_body=ExpenseCategoryReorderRequest,responses((status=204),(status=400)),tag="支出カテゴリ")]
pub async fn reorder(
    State(s): State<AppState>,
    Json(v): Json<ExpenseCategoryReorderRequest>,
) -> AppResult<StatusCode> {
    s.expense_categories.reorder(&v).await?;
    Ok(StatusCode::NO_CONTENT)
}
