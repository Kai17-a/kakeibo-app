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
}
