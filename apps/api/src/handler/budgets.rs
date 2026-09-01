use crate::{
    model::budgets::{Budget, BudgetUpsertRequest},
    service::budgets::BudgetService,
    utils::error::AppResult,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
#[derive(Clone)]
pub struct AppState {
    pub budgets: BudgetService,
}
#[utoipa::path(get,path="/api/budgets",responses((status=200,body=Vec<Budget>)))]
pub async fn list(State(s): State<AppState>) -> AppResult<Json<Vec<Budget>>> {
    Ok(Json(s.budgets.list().await?))
}
#[utoipa::path(get,path="/api/budgets/{id}",params(("id"=String,Path)),responses((status=200,body=Budget),(status=404)))]
pub async fn get(State(s): State<AppState>, Path(id): Path<String>) -> AppResult<Json<Budget>> {
    Ok(Json(s.budgets.get(&id).await?))
}
#[utoipa::path(post,path="/api/budgets",request_body=BudgetUpsertRequest,responses((status=201,body=Budget),(status=400)))]
pub async fn create(
    State(s): State<AppState>,
    Json(v): Json<BudgetUpsertRequest>,
) -> AppResult<(StatusCode, Json<Budget>)> {
    Ok((StatusCode::CREATED, Json(s.budgets.create(&v).await?)))
}
#[utoipa::path(put,path="/api/budgets/{id}",params(("id"=String,Path)),request_body=BudgetUpsertRequest,responses((status=200,body=Budget),(status=400),(status=404)))]
pub async fn update(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(v): Json<BudgetUpsertRequest>,
) -> AppResult<Json<Budget>> {
    Ok(Json(s.budgets.update(&id, &v).await?))
}
#[utoipa::path(delete,path="/api/budgets/{id}",params(("id"=String,Path)),responses((status=204),(status=404)))]
pub async fn delete(State(s): State<AppState>, Path(id): Path<String>) -> AppResult<StatusCode> {
    s.budgets.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
