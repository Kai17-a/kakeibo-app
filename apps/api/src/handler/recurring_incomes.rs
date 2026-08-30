use crate::{
    model::recurring_incomes::{RecurringIncome, RecurringIncomeUpsertRequest},
    service::recurring_incomes::RecurringIncomeService,
    utils::error::AppResult,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

#[derive(Clone)]
pub struct AppState {
    pub recurring_incomes: RecurringIncomeService,
}

#[utoipa::path(get,path="/api/recurring-incomes",responses((status=200,body=Vec<RecurringIncome>)))]
pub async fn list(State(s): State<AppState>) -> AppResult<Json<Vec<RecurringIncome>>> {
    Ok(Json(s.recurring_incomes.list().await?))
}
#[utoipa::path(get,path="/api/recurring-incomes/{id}",params(("id"=String,Path)),responses((status=200,body=RecurringIncome),(status=404)))]
pub async fn get(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<RecurringIncome>> {
    Ok(Json(s.recurring_incomes.get(&id).await?))
}
#[utoipa::path(post,path="/api/recurring-incomes",request_body=RecurringIncomeUpsertRequest,responses((status=201,body=RecurringIncome),(status=400)))]
pub async fn create(
    State(s): State<AppState>,
    Json(v): Json<RecurringIncomeUpsertRequest>,
) -> AppResult<(StatusCode, Json<RecurringIncome>)> {
    Ok((
        StatusCode::CREATED,
        Json(s.recurring_incomes.create(&v).await?),
    ))
}
#[utoipa::path(put,path="/api/recurring-incomes/{id}",params(("id"=String,Path)),request_body=RecurringIncomeUpsertRequest,responses((status=200,body=RecurringIncome),(status=400),(status=404)))]
pub async fn update(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(v): Json<RecurringIncomeUpsertRequest>,
) -> AppResult<Json<RecurringIncome>> {
    Ok(Json(s.recurring_incomes.update(&id, &v).await?))
}
#[utoipa::path(delete,path="/api/recurring-incomes/{id}",params(("id"=String,Path)),responses((status=204),(status=404)))]
pub async fn delete(State(s): State<AppState>, Path(id): Path<String>) -> AppResult<StatusCode> {
    s.recurring_incomes.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
