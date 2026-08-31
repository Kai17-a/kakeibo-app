use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, FromRow, ToSchema)]
pub struct IncomeCategoryRow {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_category_id: Option<String>,
}
