use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct BudgetRow {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub category_id: String,
    pub amount: String,
}
