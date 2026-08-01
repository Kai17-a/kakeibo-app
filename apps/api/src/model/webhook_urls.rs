use crate::database::models::webhook_urls::WebhookUrlRow;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WebhookUrl {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub url: String,
    pub description: Option<String>,
    pub is_active: bool,
}
impl From<WebhookUrlRow> for WebhookUrl {
    fn from(v: WebhookUrlRow) -> Self {
        Self {
            id: v.id,
            created_at: v.created_at,
            updated_at: v.updated_at,
            url: v.url,
            description: v.description,
            is_active: v.is_active,
        }
    }
}
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct WebhookUrlUpsertRequest {
    pub url: String,
    pub description: Option<String>,
    pub is_active: bool,
}
