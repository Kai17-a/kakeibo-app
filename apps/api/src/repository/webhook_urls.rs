use crate::{
    database::models::webhook_urls::WebhookUrlRow, model::webhook_urls::WebhookUrlUpsertRequest,
    utils::error::AppResult,
};
use sqlx::SqlitePool;
#[derive(Clone)]
pub struct WebhookUrlRepository {
    pool: SqlitePool,
}
impl WebhookUrlRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    pub async fn find_all(&self) -> AppResult<Vec<WebhookUrlRow>> {
        sqlx::query_as(include_str!("../../queries/webhook_urls/find_all.sql"))
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }
    pub async fn find_active(&self) -> AppResult<Vec<WebhookUrlRow>> {
        sqlx::query_as(include_str!("../../queries/webhook_urls/find_active.sql"))
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }
    pub async fn find_by_id(&self, id: &str) -> AppResult<Option<WebhookUrlRow>> {
        sqlx::query_as(include_str!("../../queries/webhook_urls/find_by_id.sql"))
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }
    pub async fn insert(&self, v: &WebhookUrlUpsertRequest) -> AppResult<WebhookUrlRow> {
        sqlx::query_as(include_str!("../../queries/webhook_urls/insert.sql"))
            .bind(&v.url)
            .bind(&v.description)
            .bind(v.is_active)
            .fetch_one(&self.pool)
            .await
            .map_err(Into::into)
    }
    pub async fn update(
        &self,
        id: &str,
        v: &WebhookUrlUpsertRequest,
    ) -> AppResult<Option<WebhookUrlRow>> {
        sqlx::query_as(include_str!("../../queries/webhook_urls/update.sql"))
            .bind(&v.url)
            .bind(&v.description)
            .bind(v.is_active)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }
    pub async fn delete(&self, id: &str) -> AppResult<bool> {
        Ok(
            sqlx::query(include_str!("../../queries/webhook_urls/delete_by_id.sql"))
                .bind(id)
                .execute(&self.pool)
                .await?
                .rows_affected()
                > 0,
        )
    }
}
