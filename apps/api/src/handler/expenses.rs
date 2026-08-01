use crate::{
    model::expenses::{Expense, ExpenseUpsertRequest},
    service::{expenses::ExpenseService, webhook_urls::WebhookUrlService},
    utils::error::AppResult,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
#[derive(Clone)]
pub struct AppState {
    pub expenses: ExpenseService,
    pub webhook_urls: WebhookUrlService,
}
#[utoipa::path(get,path="/api/expenses",responses((status=200,body=Vec<Expense>)))]
pub async fn list(State(s): State<AppState>) -> AppResult<Json<Vec<Expense>>> {
    Ok(Json(s.expenses.list().await?))
}
#[utoipa::path(get,path="/api/expenses/{id}",params(("id"=String,Path)),responses((status=200,body=Expense),(status=404)))]
pub async fn get(State(s): State<AppState>, Path(id): Path<String>) -> AppResult<Json<Expense>> {
    Ok(Json(s.expenses.get(&id).await?))
}
#[utoipa::path(post,path="/api/expenses",request_body=ExpenseUpsertRequest,responses((status=201,body=Expense),(status=400)))]
pub async fn create(
    State(s): State<AppState>,
    Json(v): Json<ExpenseUpsertRequest>,
) -> AppResult<(StatusCode, Json<Expense>)> {
    let expense = s.expenses.create(&v).await?;
    s.webhook_urls
        .notify(
            "expense.created",
            serde_json::to_value(&expense).unwrap_or_default(),
        )
        .await;
    Ok((StatusCode::CREATED, Json(expense)))
}
#[utoipa::path(put,path="/api/expenses/{id}",params(("id"=String,Path)),request_body=ExpenseUpsertRequest,responses((status=200,body=Expense),(status=400),(status=404)))]
pub async fn update(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(v): Json<ExpenseUpsertRequest>,
) -> AppResult<Json<Expense>> {
    Ok(Json(s.expenses.update(&id, &v).await?))
}
#[utoipa::path(delete,path="/api/expenses/{id}",params(("id"=String,Path)),responses((status=204),(status=404)))]
pub async fn delete(State(s): State<AppState>, Path(id): Path<String>) -> AppResult<StatusCode> {
    s.expenses.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
