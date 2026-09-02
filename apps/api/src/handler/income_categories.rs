use axum::{
    Json,
    extract::{Path, Query, State, rejection::QueryRejection},
    http::StatusCode,
};

use crate::{
    model::income_categories::{
        IncomeCategory, IncomeCategoryListResponse, IncomeCategoryQuery,
        IncomeCategoryReorderRequest, IncomeCategoryUpsertRequest,
    },
    service::income_categories::IncomeCategoryService,
    utils::error::{AppError, AppResult},
};

#[derive(Clone)]
pub struct AppState {
    pub income_categories: IncomeCategoryService,
}

#[utoipa::path(
    get,
    path = "/api/income-categories",
    params(IncomeCategoryQuery),
    responses((status = 200, body = IncomeCategoryListResponse), (status = 400))
)]
pub async fn list(
    State(state): State<AppState>,
    query: Result<Query<IncomeCategoryQuery>, QueryRejection>,
) -> AppResult<Json<IncomeCategoryListResponse>> {
    let Query(query) = query
        .map_err(|error| AppError::bad_request(&format!("Invalid query parameters: {error}")))?;
    Ok(Json(state.income_categories.list(&query).await?))
}

#[utoipa::path(get, path = "/api/income-categories/{id}", params(("id" = String, Path)), responses((status = 200, body = IncomeCategory), (status = 404)))]
pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<IncomeCategory>> {
    Ok(Json(state.income_categories.get(&id).await?))
}

#[utoipa::path(post, path = "/api/income-categories", request_body = IncomeCategoryUpsertRequest, responses((status = 201, body = IncomeCategory), (status = 400)))]
pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<IncomeCategoryUpsertRequest>,
) -> AppResult<(StatusCode, Json<IncomeCategory>)> {
    Ok((
        StatusCode::CREATED,
        Json(state.income_categories.create(&input).await?),
    ))
}

#[utoipa::path(put, path = "/api/income-categories/{id}", params(("id" = String, Path)), request_body = IncomeCategoryUpsertRequest, responses((status = 200, body = IncomeCategory), (status = 400), (status = 404)))]
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<IncomeCategoryUpsertRequest>,
) -> AppResult<Json<IncomeCategory>> {
    Ok(Json(state.income_categories.update(&id, &input).await?))
}

#[utoipa::path(delete, path = "/api/income-categories/{id}", params(("id" = String, Path)), responses((status = 204), (status = 404)))]
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    state.income_categories.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
#[utoipa::path(put, path = "/api/income-categories/order", request_body = IncomeCategoryReorderRequest, responses((status = 204), (status = 400)), tag = "収入カテゴリ")]
pub async fn reorder(
    State(state): State<AppState>,
    Json(input): Json<IncomeCategoryReorderRequest>,
) -> AppResult<StatusCode> {
    state.income_categories.reorder(&input).await?;
    Ok(StatusCode::NO_CONTENT)
}
