use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::database::models::income_categories::IncomeCategoryRow;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IncomeCategory {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_category_id: Option<String>,
    pub display_order: i64,
}

impl From<IncomeCategoryRow> for IncomeCategory {
    fn from(row: IncomeCategoryRow) -> Self {
        Self {
            id: row.id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            name: row.name,
            description: row.description,
            parent_category_id: row.parent_category_id,
            display_order: row.display_order,
        }
    }
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct IncomeCategoryUpsertRequest {
    pub name: String,
    pub description: Option<String>,
    pub parent_category_id: Option<String>,
}
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct IncomeCategoryReorderRequest {
    pub parent_category_id: Option<String>,
    pub category_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct IncomeCategoryListResponse {
    pub items: Vec<IncomeCategory>,
    pub pagination: IncomeCategoryPagination,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct IncomeCategoryPagination {
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub struct IncomeCategoryQuery {
    pub id: Option<String>,
    pub sort_by: Option<IncomeCategorySortBy>,
    pub sort_order: Option<IncomeCategorySortOrder>,
    #[param(default = 1, minimum = 1)]
    pub page: Option<u32>,
    #[param(default = 50, minimum = 1, maximum = 100)]
    pub per_page: Option<u32>,
}

impl IncomeCategoryQuery {
    pub(crate) fn page(&self) -> u32 {
        self.page.unwrap_or(1)
    }

    pub(crate) fn per_page(&self) -> u32 {
        self.per_page.unwrap_or(50)
    }
}

#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum IncomeCategorySortBy {
    Id,
    Name,
    CreatedAt,
    UpdatedAt,
    DisplayOrder,
}

impl IncomeCategorySortBy {
    pub(crate) fn sql(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Name => "name",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
            Self::DisplayOrder => "display_order",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum IncomeCategorySortOrder {
    Asc,
    Desc,
}

impl IncomeCategorySortOrder {
    pub(crate) fn sql(self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}
