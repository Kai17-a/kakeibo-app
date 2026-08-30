use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::database::models::incomes::IncomeRow;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Income {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub category_id: String,
    pub transaction_date: String,
    pub amount: String,
    pub recurring_income_id: Option<String>,
    pub description: Option<String>,
}

impl From<IncomeRow> for Income {
    fn from(row: IncomeRow) -> Self {
        Self {
            id: row.id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            category_id: row.category_id,
            transaction_date: row.transaction_date,
            amount: row.amount,
            recurring_income_id: row.recurring_income_id,
            description: row.description,
        }
    }
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct IncomeUpsertRequest {
    pub category_id: String,
    pub transaction_date: String,
    pub amount: String,
    pub recurring_income_id: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct IncomeListResponse {
    pub items: Vec<Income>,
    pub pagination: Pagination,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Clone, Default, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub struct IncomeQuery {
    pub id: Option<String>,
    pub category_id: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub sort_by: Option<IncomeSortBy>,
    pub sort_order: Option<SortOrder>,
    #[param(default = 1, minimum = 1)]
    pub page: Option<u32>,
    #[param(default = 50, minimum = 1, maximum = 100)]
    pub per_page: Option<u32>,
}

impl IncomeQuery {
    pub(crate) fn page(&self) -> u32 {
        self.page.unwrap_or(1)
    }

    pub(crate) fn per_page(&self) -> u32 {
        self.per_page.unwrap_or(50)
    }
}

#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum IncomeSortBy {
    Id,
    CategoryId,
    TransactionDate,
    Amount,
    CreatedAt,
    UpdatedAt,
}

impl IncomeSortBy {
    pub(crate) fn sql(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::CategoryId => "category_id",
            Self::TransactionDate => "transaction_date",
            Self::Amount => "CAST(amount AS NUMERIC)",
            Self::CreatedAt => "created_at",
            Self::UpdatedAt => "updated_at",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    Asc,
    Desc,
}

impl SortOrder {
    pub(crate) fn sql(self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}
