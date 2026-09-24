use axum::{
    Json,
    extract::{Path, Query, State, rejection::QueryRejection},
    http::StatusCode,
};

use crate::{
    model::incomes::{Income, IncomeListResponse, IncomeQuery, IncomeUpsertRequest},
    service::incomes::IncomeService,
    utils::error::AppResult,
};

#[derive(Clone)]
pub struct AppState {
    pub incomes: IncomeService,
}

#[utoipa::path(
    get,
    path = "/api/incomes",
    params(IncomeQuery),
    responses((status = 200, body = IncomeListResponse), (status = 400))
)]
pub async fn list(
    State(state): State<AppState>,
    query: Result<Query<IncomeQuery>, QueryRejection>,
) -> AppResult<Json<IncomeListResponse>> {
    let Query(query) = query.map_err(|error| {
        crate::utils::error::AppError::bad_request(&format!("Invalid query parameters: {error}"))
    })?;
    Ok(Json(state.incomes.list(&query).await?))
}

#[utoipa::path(get, path = "/api/incomes/{id}", params(("id" = String, Path)), responses((status = 200, body = Income), (status = 404)))]
pub async fn get(State(state): State<AppState>, Path(id): Path<String>) -> AppResult<Json<Income>> {
    Ok(Json(state.incomes.get(&id).await?))
}

#[utoipa::path(post, path = "/api/incomes", request_body = IncomeUpsertRequest, responses((status = 201, body = Income)))]
pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<IncomeUpsertRequest>,
) -> AppResult<(StatusCode, Json<Income>)> {
    Ok((
        StatusCode::CREATED,
        Json(state.incomes.create(&input).await?),
    ))
}

#[utoipa::path(put, path = "/api/incomes/{id}", params(("id" = String, Path)), request_body = IncomeUpsertRequest, responses((status = 200, body = Income), (status = 404)))]
pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<IncomeUpsertRequest>,
) -> AppResult<Json<Income>> {
    Ok(Json(state.incomes.update(&id, &input).await?))
}

#[utoipa::path(delete, path = "/api/incomes/{id}", params(("id" = String, Path)), responses((status = 204), (status = 404)))]
pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    state.incomes.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
