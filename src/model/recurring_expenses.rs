use crate::database::models::recurring_expenses::RecurringExpenseRow;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RecurringExpense {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub amount: String,
    pub payment_day: i64,
    pub start_date: String,
    pub end_date: Option<String>,
    pub category_id: String,
    pub payment_method_id: String,
    pub is_active: bool,
    pub description: Option<String>,
}
impl From<RecurringExpenseRow> for RecurringExpense {
    fn from(v: RecurringExpenseRow) -> Self {
        Self {
            id: v.id,
            created_at: v.created_at,
            updated_at: v.updated_at,
            name: v.name,
            amount: v.amount,
            payment_day: v.payment_day,
            start_date: v.start_date,
            end_date: v.end_date,
            category_id: v.category_id,
            payment_method_id: v.payment_method_id,
            is_active: v.is_active,
            description: v.description,
        }
    }
}
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct RecurringExpenseUpsertRequest {
    pub name: String,
    pub amount: String,
    pub payment_day: i64,
    pub start_date: String,
    pub end_date: Option<String>,
    pub category_id: String,
    pub payment_method_id: String,
    pub is_active: bool,
    pub description: Option<String>,
}
