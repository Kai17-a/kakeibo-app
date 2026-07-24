use crate::{model::import::ImportResult, service::import::ImportService, utils::error::AppResult};
use axum::{Json, extract::State, http::StatusCode};
#[derive(Clone)]
pub struct AppState {
    pub import: ImportService,
}
#[utoipa::path(post,path="/api/import/expenses",request_body(content=String,content_type="text/csv"),responses((status=201,body=ImportResult),(status=400)))]
pub async fn expenses(
    State(s): State<AppState>,
    body: String,
) -> AppResult<(StatusCode, Json<ImportResult>)> {
    Ok((
        StatusCode::CREATED,
        Json(s.import.import_expenses(&body).await?),
    ))
}
#[utoipa::path(post,path="/api/import/incomes",request_body(content=String,content_type="text/csv"),responses((status=201,body=ImportResult),(status=400)))]
pub async fn incomes(
    State(s): State<AppState>,
    body: String,
) -> AppResult<(StatusCode, Json<ImportResult>)> {
    Ok((
        StatusCode::CREATED,
        Json(s.import.import_incomes(&body).await?),
    ))
}
