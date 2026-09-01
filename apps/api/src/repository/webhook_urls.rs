use crate::{
    database::models::webhook_urls::WebhookUrlRow,
    model::webhook_urls::{WebhookEvent, WebhookUrl, WebhookUrlUpsertRequest},
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
    pub async fn find_all(&self) -> AppResult<Vec<WebhookUrl>> {
        let rows: Vec<WebhookUrlRow> =
            sqlx::query_as(include_str!("../../queries/webhook_urls/find_all.sql"))
                .fetch_all(&self.pool)
                .await?;
        self.with_events(rows).await
    }
    pub async fn find_active(&self) -> AppResult<Vec<WebhookUrl>> {
        let rows: Vec<WebhookUrlRow> =
            sqlx::query_as(include_str!("../../queries/webhook_urls/find_active.sql"))
                .fetch_all(&self.pool)
                .await?;
        self.with_events(rows).await
    }
    pub async fn find_by_id(&self, id: &str) -> AppResult<Option<WebhookUrl>> {
        let row: Option<WebhookUrlRow> =
            sqlx::query_as(include_str!("../../queries/webhook_urls/find_by_id.sql"))
                .bind(id)
                .fetch_optional(&self.pool)
                .await?;
        match row {
            Some(row) => Ok(Some(WebhookUrl::from_row(row, self.find_events(id).await?))),
            None => Ok(None),
        }
    }
    pub async fn insert(&self, v: &WebhookUrlUpsertRequest) -> AppResult<WebhookUrl> {
        let mut tx = self.pool.begin().await?;
        let row: WebhookUrlRow =
            sqlx::query_as(include_str!("../../queries/webhook_urls/insert.sql"))
                .bind(&v.url)
                .bind(&v.description)
                .bind(v.is_active)
                .fetch_one(&mut *tx)
                .await?;
        for event in &v.events {
            sqlx::query("INSERT INTO webhook_url_events (webhook_url_id, event) VALUES (?, ?)")
                .bind(&row.id)
                .bind(event.as_str())
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(WebhookUrl::from_row(row, v.events.clone()))
    }
    pub async fn update(
        &self,
        id: &str,
        v: &WebhookUrlUpsertRequest,
    ) -> AppResult<Option<WebhookUrl>> {
        let mut tx = self.pool.begin().await?;
        let row: Option<WebhookUrlRow> =
            sqlx::query_as(include_str!("../../queries/webhook_urls/update.sql"))
                .bind(&v.url)
                .bind(&v.description)
                .bind(v.is_active)
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?;
        let Some(row) = row else {
            tx.rollback().await?;
            return Ok(None);
        };
        sqlx::query("DELETE FROM webhook_url_events WHERE webhook_url_id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        for event in &v.events {
            sqlx::query("INSERT INTO webhook_url_events (webhook_url_id, event) VALUES (?, ?)")
                .bind(id)
                .bind(event.as_str())
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(Some(WebhookUrl::from_row(row, v.events.clone())))
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

    async fn find_events(&self, id: &str) -> AppResult<Vec<WebhookEvent>> {
        let events: Vec<String> = sqlx::query_scalar(
            "SELECT event FROM webhook_url_events WHERE webhook_url_id = ? ORDER BY event ASC",
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await?;
        events.into_iter().map(WebhookEvent::try_from).collect()
    }

    async fn with_events(&self, rows: Vec<WebhookUrlRow>) -> AppResult<Vec<WebhookUrl>> {
        let mut urls = Vec::with_capacity(rows.len());
        for row in rows {
            let events = self.find_events(&row.id).await?;
            urls.push(WebhookUrl::from_row(row, events));
        }
        Ok(urls)
    }
}
