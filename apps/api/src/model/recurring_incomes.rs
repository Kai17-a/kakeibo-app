use crate::database::models::recurring_incomes::RecurringIncomeRow;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RecurringIncome {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub amount: String,
    pub payment_day: i64,
    pub start_date: String,
    pub end_date: Option<String>,
    pub category_id: String,
    pub is_active: bool,
    pub is_variable: bool,
    pub description: Option<String>,
}

impl From<RecurringIncomeRow> for RecurringIncome {
    fn from(v: RecurringIncomeRow) -> Self {
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
            is_active: v.is_active,
            is_variable: v.is_variable,
            description: v.description,
        }
    }
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct RecurringIncomeUpsertRequest {
    pub name: String,
    pub amount: String,
    pub payment_day: i64,
    pub start_date: String,
    pub end_date: Option<String>,
    pub category_id: String,
    pub is_active: bool,
    pub is_variable: bool,
    pub description: Option<String>,
}
