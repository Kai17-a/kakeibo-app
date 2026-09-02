use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct ExpenseCategoryRow {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_category_id: Option<String>,
    pub display_order: i64,
}
