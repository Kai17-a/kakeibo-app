use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct ExpenseCategoryRow {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub description: Option<String>,
}
