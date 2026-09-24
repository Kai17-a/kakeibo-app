use crate::database::models::recurring_expenses::RecurringExpenseRow;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ExchangeRatePreview {
    pub foreign_amount: String,
    pub currency_code: String,
    pub exchange_rate: String,
    pub exchange_rate_date: String,
    pub converted_amount: String,
}
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PendingMonthsResponse {
    pub months: Vec<String>,
}
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct BackfillResponse {
    pub created: Vec<String>,
    pub skipped: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RecurringExpense {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub amount: Option<String>,
    pub payment_day: i64,
    pub start_date: String,
    pub end_date: Option<String>,
    pub category_id: String,
    pub payment_method_id: String,
    pub is_active: bool,
    pub is_variable: bool,
    pub description: Option<String>,
    pub foreign_amount: Option<String>,
    pub currency_code: Option<String>,
    pub exchange_rate: Option<String>,
}
impl From<RecurringExpenseRow> for RecurringExpense {
    fn from(v: RecurringExpenseRow) -> Self {
        Self {
            id: v.id,
            created_at: v.created_at,
            updated_at: v.updated_at,
            name: v.name,
            amount: (!v.amount.is_empty()).then_some(v.amount),
            payment_day: v.payment_day,
            start_date: v.start_date,
            end_date: v.end_date,
            category_id: v.category_id,
            payment_method_id: v.payment_method_id,
            is_active: v.is_active,
            is_variable: v.is_variable,
            description: v.description,
            foreign_amount: v.foreign_amount,
            currency_code: v.currency_code,
            exchange_rate: v.exchange_rate,
        }
    }
}
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct RecurringExpenseUpsertRequest {
    pub name: String,
    #[serde(default)]
    pub amount: Option<String>,
    pub payment_day: i64,
    pub start_date: String,
    pub end_date: Option<String>,
    pub category_id: String,
    pub payment_method_id: String,
    pub is_active: bool,
    pub is_variable: bool,
    pub description: Option<String>,
    #[serde(default)]
    pub foreign_amount: Option<String>,
    #[serde(default)]
    pub currency_code: Option<String>,
    #[serde(default)]
    pub exchange_rate: Option<String>,
    #[serde(default)]
    pub sync_future_transactions: bool,
}
