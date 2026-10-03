use crate::database::models::transfers::TransferRow;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Transfer {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub transaction_date: String,
    pub amount: String,
    pub from_payment_method_id: String,
    pub to_payment_method_id: String,
    pub description: Option<String>,
}

impl From<TransferRow> for Transfer {
    fn from(value: TransferRow) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at,
            updated_at: value.updated_at,
            transaction_date: value.transaction_date,
            amount: value.amount,
            from_payment_method_id: value.from_payment_method_id,
            to_payment_method_id: value.to_payment_method_id,
            description: value.description,
        }
    }
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct TransferUpsertRequest {
    pub transaction_date: String,
    pub amount: String,
    pub from_payment_method_id: String,
    pub to_payment_method_id: String,
    pub description: Option<String>,
}
