use crate::database::models::budgets::BudgetRow;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Budget {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub category_id: String,
    pub amount: String,
}

impl From<BudgetRow> for Budget {
    fn from(v: BudgetRow) -> Self {
        Self {
            id: v.id,
            created_at: v.created_at,
            updated_at: v.updated_at,
            category_id: v.category_id,
            amount: v.amount,
        }
    }
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct BudgetUpsertRequest {
    pub category_id: String,
    pub amount: String,
}
