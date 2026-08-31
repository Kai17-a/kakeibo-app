use crate::database::models::payment_methods::PaymentMethodRow;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaymentMethod {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub name: String,
    pub description: Option<String>,
    pub initial_balance: Option<String>,
    pub balance: Option<String>,
}
impl TryFrom<PaymentMethodRow> for PaymentMethod {
    type Error = crate::utils::error::AppError;

    fn try_from(v: PaymentMethodRow) -> Result<Self, Self::Error> {
        let balance = v
            .initial_balance
            .as_deref()
            .map(|initial_balance| {
                let initial_balance = initial_balance.parse::<i64>().map_err(|error| {
                    crate::utils::error::AppError::context("Invalid initial balance", error)
                })?;
                initial_balance
                    .checked_add(v.income_total)
                    .and_then(|value| value.checked_sub(v.expense_total))
                    .map(|value| value.to_string())
                    .ok_or_else(|| {
                        crate::utils::error::AppError::bad_request("balance is outside i64 range")
                    })
            })
            .transpose()?;
        Ok(Self {
            id: v.id,
            created_at: v.created_at,
            updated_at: v.updated_at,
            name: v.name,
            description: v.description,
            initial_balance: v.initial_balance,
            balance,
        })
    }
}
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct PaymentMethodUpsertRequest {
    pub name: String,
    pub description: Option<String>,
    pub initial_balance: Option<String>,
}
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PaymentMethodListResponse {
    pub items: Vec<PaymentMethod>,
    pub pagination: PaymentMethodPagination,
}
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PaymentMethodPagination {
    pub page: u32,
    pub per_page: u32,
}
#[derive(Debug, Clone, Default, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
#[into_params(parameter_in=Query)]
pub struct PaymentMethodQuery {
    pub id: Option<String>,
    pub sort_by: Option<PaymentMethodSortBy>,
    pub sort_order: Option<PaymentMethodSortOrder>,
    #[param(default = 1, minimum = 1)]
    pub page: Option<u32>,
    #[param(default = 50, minimum = 1, maximum = 100)]
    pub per_page: Option<u32>,
}
impl PaymentMethodQuery {
    pub(crate) fn page(&self) -> u32 {
        self.page.unwrap_or(1)
    }
    pub(crate) fn per_page(&self) -> u32 {
        self.per_page.unwrap_or(50)
    }
}
#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethodSortBy {
    Id,
    Name,
    CreatedAt,
    UpdatedAt,
}
impl PaymentMethodSortBy {
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
pub enum PaymentMethodSortOrder {
    Asc,
    Desc,
}
impl PaymentMethodSortOrder {
    pub(crate) fn sql(self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}
