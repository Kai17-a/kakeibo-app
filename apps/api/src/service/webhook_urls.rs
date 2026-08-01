use crate::{
    model::webhook_urls::{WebhookUrl, WebhookUrlUpsertRequest},
    repository::webhook_urls::WebhookUrlRepository,
    utils::error::{AppError, AppResult},
};
#[derive(Clone)]
pub struct WebhookUrlService {
    repository: WebhookUrlRepository,
}
impl WebhookUrlService {
    pub fn new(repository: WebhookUrlRepository) -> Self {
        Self { repository }
    }
    pub async fn list(&self) -> AppResult<Vec<WebhookUrl>> {
        Ok(self
            .repository
            .find_all()
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }
    pub async fn get(&self, id: &str) -> AppResult<WebhookUrl> {
        self.repository
            .find_by_id(id)
            .await?
            .map(Into::into)
            .ok_or_else(|| AppError::not_found("webhook url", id))
    }
    pub async fn create(&self, v: &WebhookUrlUpsertRequest) -> AppResult<WebhookUrl> {
        validate(v)?;
        self.repository.insert(v).await.map(Into::into)
    }
    pub async fn update(&self, id: &str, v: &WebhookUrlUpsertRequest) -> AppResult<WebhookUrl> {
        validate(v)?;
        self.repository
            .update(id, v)
            .await?
            .map(Into::into)
            .ok_or_else(|| AppError::not_found("webhook url", id))
    }
    pub async fn delete(&self, id: &str) -> AppResult<()> {
        if self.repository.delete(id).await? {
            Ok(())
        } else {
            Err(AppError::not_found("webhook url", id))
        }
    }
    /// 有効なすべてのWebhook URLへイベントを非同期で通知する。
    /// 通知の失敗はログに記録するだけで、呼び出し元の処理には影響させない。
    pub async fn notify(&self, event: &str, data: serde_json::Value) {
        let urls = match self.repository.find_active().await {
            Ok(urls) => urls,
            Err(error) => {
                tracing::warn!(%error, "Failed to load webhook urls");
                return;
            }
        };
        if urls.is_empty() {
            return;
        }
        let client = reqwest::Client::new();
        let payload = serde_json::json!({ "event": event, "data": data });
        for row in urls {
            let client = client.clone();
            let payload = payload.clone();
            let url = row.url.clone();
            tokio::spawn(async move {
                let result = tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    client.post(&url).json(&payload).send(),
                )
                .await;
                match result {
                    Ok(Ok(response)) if response.status().is_success() => {}
                    Ok(Ok(response)) => {
                        tracing::warn!(%url, status = %response.status(), "Webhook notification failed")
                    }
                    Ok(Err(error)) => tracing::warn!(%url, %error, "Webhook notification failed"),
                    Err(_) => tracing::warn!(%url, "Webhook notification timed out"),
                }
            });
        }
    }
}
fn validate(v: &WebhookUrlUpsertRequest) -> AppResult<()> {
    let url = v.url.trim();
    if url.is_empty() {
        return Err(AppError::bad_request("url must not be empty"));
    }
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(AppError::bad_request(
            "url must start with http:// or https://",
        ));
    }
    Ok(())
}
