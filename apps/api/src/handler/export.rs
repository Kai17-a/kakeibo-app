use crate::{service::export::ExportService, utils::error::AppResult};
use axum::{extract::State, http::header, response::IntoResponse};
#[derive(Clone)]
pub struct AppState {
    pub export: ExportService,
}
#[utoipa::path(get,path="/api/export/expenses",responses((status=200,description="支出のCSVファイル",content_type="text/csv")))]
pub async fn expenses(State(s): State<AppState>) -> AppResult<impl IntoResponse> {
    Ok(csv_response("expenses.csv", s.export.expenses_csv().await?))
}
#[utoipa::path(get,path="/api/export/incomes",responses((status=200,description="収入のCSVファイル",content_type="text/csv")))]
pub async fn incomes(State(s): State<AppState>) -> AppResult<impl IntoResponse> {
    Ok(csv_response("incomes.csv", s.export.incomes_csv().await?))
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
