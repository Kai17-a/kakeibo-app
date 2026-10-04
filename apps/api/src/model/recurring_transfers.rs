use crate::database::models::recurring_transfers::RecurringTransferRow;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct RecurringTransfer {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub amount: String,
    pub payment_day: i64,
    pub start_date: String,
    pub end_date: Option<String>,
    pub from_payment_method_id: String,
    pub to_payment_method_id: String,
    pub is_active: bool,
    pub description: Option<String>,
}

impl From<RecurringTransferRow> for RecurringTransfer {
    fn from(value: RecurringTransferRow) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at,
            updated_at: value.updated_at,
            name: value.name,
            amount: value.amount,
            payment_day: value.payment_day,
            start_date: value.start_date,
            end_date: value.end_date,
            from_payment_method_id: value.from_payment_method_id,
            to_payment_method_id: value.to_payment_method_id,
            is_active: value.is_active,
            description: value.description,
        }
    }
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct RecurringTransferUpsertRequest {
    pub name: String,
    pub amount: String,
    pub payment_day: i64,
    pub start_date: String,
    pub end_date: Option<String>,
    pub from_payment_method_id: String,
    pub to_payment_method_id: String,
    pub is_active: bool,
    pub description: Option<String>,
}
