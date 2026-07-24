use sqlx::FromRow;
#[derive(Debug, Clone, FromRow)]
pub struct ExpenseExportRow {
    pub transaction_date: String,
    pub amount: String,
    pub category_name: String,
    pub payment_method_name: String,
    pub description: Option<String>,
}
#[derive(Debug, Clone, FromRow)]
pub struct IncomeExportRow {
    pub transaction_date: String,
    pub amount: String,
    pub category_name: String,
    pub description: Option<String>,
}
