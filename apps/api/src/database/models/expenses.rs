use sqlx::FromRow;
#[derive(Debug, Clone, FromRow)]
pub struct ExpenseRow {
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
