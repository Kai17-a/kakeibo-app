use sqlx::FromRow;
#[derive(Debug, Clone, FromRow)]
pub struct RecurringExpenseRow {
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
