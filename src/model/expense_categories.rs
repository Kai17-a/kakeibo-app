use crate::database::models::expense_categories::ExpenseCategoryRow;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExpenseCategory {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub description: Option<String>,
}
impl From<ExpenseCategoryRow> for ExpenseCategory {
    fn from(v: ExpenseCategoryRow) -> Self {
        Self {
            id: v.id,
            created_at: v.created_at,
            updated_at: v.updated_at,
            name: v.name,
            description: v.description,
        }
    }
}
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ExpenseCategoryUpsertRequest {
    pub name: String,
    pub description: Option<String>,
}
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ExpenseCategoryListResponse {
    pub items: Vec<ExpenseCategory>,
    pub pagination: ExpenseCategoryPagination,
}
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ExpenseCategoryPagination {
    pub page: u32,
    pub per_page: u32,
}
#[derive(Debug, Clone, Default, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub struct ExpenseCategoryQuery {
    pub id: Option<String>,
    pub sort_by: Option<ExpenseCategorySortBy>,
    pub sort_order: Option<ExpenseCategorySortOrder>,
    #[param(default = 1, minimum = 1)]
    pub page: Option<u32>,
    #[param(default = 50, minimum = 1, maximum = 100)]
    pub per_page: Option<u32>,
}
impl ExpenseCategoryQuery {
    pub(crate) fn page(&self) -> u32 {
        self.page.unwrap_or(1)
    }
    pub(crate) fn per_page(&self) -> u32 {
        self.per_page.unwrap_or(50)
    }
}
#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExpenseCategorySortBy {
    Id,
    Name,
    CreatedAt,
    UpdatedAt,
}
impl ExpenseCategorySortBy {
    pub(crate) fn sql(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Name => "name",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }
}
#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExpenseCategorySortOrder {
    Asc,
    Desc,
}
impl ExpenseCategorySortOrder {
    pub(crate) fn sql(self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}
