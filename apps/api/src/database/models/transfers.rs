use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct TransferRow {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub transaction_date: String,
    pub amount: String,
    pub from_payment_method_id: String,
    pub to_payment_method_id: String,
    pub description: Option<String>,
}
