use sqlx::FromRow;
#[derive(Debug, Clone, FromRow)]
pub struct WebhookUrlRow {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub url: String,
    pub description: Option<String>,
    pub is_active: bool,
}
