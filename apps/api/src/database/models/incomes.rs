use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct IncomeRow {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub category_id: String,
    pub transaction_date: String,
    pub amount: String,
    pub recurring_income_id: Option<String>,
    pub description: Option<String>,
}
