use crate::{
    model::recurring_expenses::{
        ExchangeRatePreview, RecurringExpense, RecurringExpenseUpsertRequest,
    },
    service::recurring_expenses::RecurringExpenseService,
    utils::error::AppResult,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::NaiveDate;
use serde::Deserialize;
#[derive(Clone)]
pub struct AppState {
    pub recurring_expenses: RecurringExpenseService,
}
#[derive(Debug, Deserialize)]
pub struct ExchangeRateQuery {
    pub month: String,
}
#[utoipa::path(get,path="/api/recurring-expenses",responses((status=200,body=Vec<RecurringExpense>)))]
pub async fn list(State(s): State<AppState>) -> AppResult<Json<Vec<RecurringExpense>>> {
    Ok(Json(s.recurring_expenses.list().await?))
}
#[utoipa::path(get,path="/api/recurring-expenses/{id}",params(("id"=String,Path)),responses((status=200,body=RecurringExpense),(status=404)))]
pub async fn get(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<RecurringExpense>> {
    Ok(Json(s.recurring_expenses.get(&id).await?))
}
#[utoipa::path(post,path="/api/recurring-expenses",request_body=RecurringExpenseUpsertRequest,responses((status=201,body=RecurringExpense),(status=400)))]
pub async fn create(
    State(s): State<AppState>,
    Json(v): Json<RecurringExpenseUpsertRequest>,
) -> AppResult<(StatusCode, Json<RecurringExpense>)> {
    Ok((
        StatusCode::CREATED,
        Json(s.recurring_expenses.create(&v).await?),
    ))
}
#[utoipa::path(put,path="/api/recurring-expenses/{id}",params(("id"=String,Path)),request_body=RecurringExpenseUpsertRequest,responses((status=200,body=RecurringExpense),(status=400),(status=404)))]
pub async fn update(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(v): Json<RecurringExpenseUpsertRequest>,
) -> AppResult<Json<RecurringExpense>> {
    Ok(Json(s.recurring_expenses.update(&id, &v).await?))
}
#[utoipa::path(delete,path="/api/recurring-expenses/{id}",params(("id"=String,Path)),responses((status=204),(status=404)))]
pub async fn delete(State(s): State<AppState>, Path(id): Path<String>) -> AppResult<StatusCode> {
    s.recurring_expenses.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(get,path="/api/recurring-expenses/{id}/exchange-rate",params(("id"=String,Path),("month"=String,Query)),responses((status=200,body=ExchangeRatePreview),(status=400),(status=404),(status=422)))]
pub async fn exchange_rate_preview(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<ExchangeRateQuery>,
) -> AppResult<Json<ExchangeRatePreview>> {
    if query.month.len() != 7
        || query.month.as_bytes().get(4) != Some(&b'-')
        || !query
            .month
            .bytes()
            .enumerate()
            .all(|(index, byte)| index == 4 || byte.is_ascii_digit())
        || NaiveDate::parse_from_str(&format!("{}-01", query.month), "%Y-%m-%d").is_err()
    {
        return Err(crate::utils::error::AppError::bad_request(
            "month must be in YYYY-MM format",
        ));
    }
    Ok(Json(
        s.recurring_expenses
            .exchange_rate_preview(&id, &query.month)
            .await?,
    ))
}
