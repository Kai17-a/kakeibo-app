use sqlx::FromRow;
#[derive(Debug, Clone, FromRow)]
pub struct PaymentMethodRow {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub description: Option<String>,
    pub initial_balance: Option<String>,
    pub income_total: i64,
    pub expense_total: i64,
}
