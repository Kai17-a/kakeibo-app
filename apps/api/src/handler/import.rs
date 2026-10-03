use crate::{
    model::import::{
        ExpenseImportPreview, ImportResult, IncomeImportPreview, RecurringExpenseImportPreview,
        TransferImportPreview, VariableExpenseImportPreview,
    },
    service::import::ImportService,
    utils::error::AppResult,
};
use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
};
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
#[utoipa::path(post,path="/api/import/expenses/preview",request_body(content=String,content_type="text/csv"),responses((status=200,body=ExpenseImportPreview),(status=400)))]
pub async fn preview_expenses(
    State(s): State<AppState>,
    body: String,
) -> AppResult<Json<ExpenseImportPreview>> {
    Ok(Json(s.import.preview_expenses(&body).await?))
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
#[utoipa::path(post,path="/api/import/incomes/preview",request_body(content=String,content_type="text/csv"),responses((status=200,body=IncomeImportPreview),(status=400)))]
pub async fn preview_incomes(
    State(s): State<AppState>,
    body: String,
) -> AppResult<Json<IncomeImportPreview>> {
    Ok(Json(s.import.preview_incomes(&body).await?))
}

#[utoipa::path(post,path="/api/import/recurring-expenses",request_body(content=String,content_type="text/csv"),responses((status=201,body=ImportResult),(status=400)))]
pub async fn recurring_expenses(
    State(s): State<AppState>,
    body: String,
) -> AppResult<(StatusCode, Json<ImportResult>)> {
    Ok((
        StatusCode::CREATED,
        Json(s.import.import_recurring_expenses(&body).await?),
    ))
}

#[utoipa::path(post,path="/api/import/recurring-expenses/preview",request_body(content=String,content_type="text/csv"),responses((status=200,body=RecurringExpenseImportPreview),(status=400)))]
pub async fn preview_recurring_expenses(
    State(s): State<AppState>,
    body: String,
) -> AppResult<Json<RecurringExpenseImportPreview>> {
    Ok(Json(s.import.preview_recurring_expenses(&body).await?))
}

#[utoipa::path(post,path="/api/import/variable-expenses",request_body(content=String,content_type="text/csv"),responses((status=201,body=ImportResult),(status=400)))]
pub async fn variable_expenses(
    State(s): State<AppState>,
    body: String,
) -> AppResult<(StatusCode, Json<ImportResult>)> {
    Ok((
        StatusCode::CREATED,
        Json(s.import.import_variable_expenses(&body).await?),
    ))
}

#[utoipa::path(post,path="/api/import/variable-expenses/preview",request_body(content=String,content_type="text/csv"),responses((status=200,body=VariableExpenseImportPreview),(status=400)))]
pub async fn preview_variable_expenses(
    State(s): State<AppState>,
    body: String,
) -> AppResult<Json<VariableExpenseImportPreview>> {
    Ok(Json(s.import.preview_variable_expenses(&body).await?))
}
#[utoipa::path(post,path="/api/import/transfers",request_body(content=String,content_type="text/csv"),responses((status=201,body=ImportResult),(status=400)))]
pub async fn transfers(
    State(s): State<AppState>,
    body: String,
) -> AppResult<(StatusCode, Json<ImportResult>)> {
    Ok((
        StatusCode::CREATED,
        Json(s.import.import_transfers(&body).await?),
    ))
}
#[utoipa::path(post,path="/api/import/transfers/preview",request_body(content=String,content_type="text/csv"),responses((status=200,body=TransferImportPreview),(status=400)))]
pub async fn preview_transfers(
    State(s): State<AppState>,
    body: String,
) -> AppResult<Json<TransferImportPreview>> {
    Ok(Json(s.import.preview_transfers(&body).await?))
}

pub async fn expense_sample() -> impl IntoResponse {
    csv_response(
        "expense_import_sample.csv",
        ImportService::expense_sample_csv(),
    )
}

pub async fn income_sample() -> impl IntoResponse {
    csv_response(
        "income_import_sample.csv",
        ImportService::income_sample_csv(),
    )
}

pub async fn recurring_expense_sample() -> impl IntoResponse {
    csv_response(
        "recurring_expense_import_sample.csv",
        ImportService::recurring_expense_sample_csv(),
    )
}

pub async fn variable_expense_sample() -> impl IntoResponse {
    csv_response(
        "variable_expense_import_sample.csv",
        ImportService::variable_expense_sample_csv(),
    )
}
pub async fn transfer_sample() -> impl IntoResponse {
    csv_response(
        "transfer_import_sample.csv",
        ImportService::transfer_sample_csv(),
    )
}

fn csv_response(filename: &str, body: String) -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8".to_owned()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        body,
    )
}
