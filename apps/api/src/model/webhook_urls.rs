use crate::{database::models::webhook_urls::WebhookUrlRow, utils::error::AppError};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum WebhookEvent {
    #[serde(rename = "expense.created")]
    ExpenseCreated,
    #[serde(rename = "income.created")]
    IncomeCreated,
    #[serde(rename = "budget.exceeded")]
    BudgetExceeded,
}

impl WebhookEvent {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExpenseCreated => "expense.created",
            Self::IncomeCreated => "income.created",
            Self::BudgetExceeded => "budget.exceeded",
        }
    }
}

impl TryFrom<String> for WebhookEvent {
    type Error = AppError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "expense.created" => Ok(Self::ExpenseCreated),
            "income.created" => Ok(Self::IncomeCreated),
            "budget.exceeded" => Ok(Self::BudgetExceeded),
            _ => Err(AppError::context(
                "Unknown webhook event stored in database",
                std::io::Error::new(std::io::ErrorKind::InvalidData, value),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WebhookUrl {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub url: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub events: Vec<WebhookEvent>,
}
impl WebhookUrl {
    pub fn from_row(v: WebhookUrlRow, events: Vec<WebhookEvent>) -> Self {
        Self {
            id: v.id,
            created_at: v.created_at,
            updated_at: v.updated_at,
            url: v.url,
            description: v.description,
            is_active: v.is_active,
            events,
        }
    }
}
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct WebhookUrlUpsertRequest {
    pub url: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub events: Vec<WebhookEvent>,
}
