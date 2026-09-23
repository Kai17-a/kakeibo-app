use crate::database::models::expenses::ExpenseRow;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Expense {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub transaction_date: String,
    pub amount: String,
    pub category_id: String,
    pub payment_method_id: String,
    pub recurring_expense_id: Option<String>,
    pub description: Option<String>,
    pub foreign_amount: Option<String>,
    pub currency_code: Option<String>,
    pub exchange_rate: Option<String>,
    pub exchange_rate_date: Option<String>,
}
impl From<ExpenseRow> for Expense {
    fn from(v: ExpenseRow) -> Self {
        Self {
            id: v.id,
            created_at: v.created_at,
            updated_at: v.updated_at,
            transaction_date: v.transaction_date,
            amount: v.amount,
            category_id: v.category_id,
            payment_method_id: v.payment_method_id,
            recurring_expense_id: v.recurring_expense_id,
            description: v.description,
            foreign_amount: v.foreign_amount,
            currency_code: v.currency_code,
            exchange_rate: v.exchange_rate,
            exchange_rate_date: v.exchange_rate_date,
        }
    }
}
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ExpenseUpsertRequest {
    pub transaction_date: String,
    pub amount: String,
    pub category_id: String,
    pub payment_method_id: String,
    pub recurring_expense_id: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub foreign_amount: Option<String>,
    #[serde(default)]
    pub currency_code: Option<String>,
    #[serde(default)]
    pub exchange_rate: Option<String>,
    #[serde(default)]
    pub exchange_rate_date: Option<String>,
}
