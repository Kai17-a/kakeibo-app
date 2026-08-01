use crate::{
    model::webhook_urls::{WebhookUrl, WebhookUrlUpsertRequest},
    service::webhook_urls::WebhookUrlService,
    utils::error::AppResult,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
#[derive(Clone)]
pub struct AppState {
    pub webhook_urls: WebhookUrlService,
}
#[utoipa::path(get,path="/api/webhook-urls",responses((status=200,body=Vec<WebhookUrl>)))]
pub async fn list(State(s): State<AppState>) -> AppResult<Json<Vec<WebhookUrl>>> {
    Ok(Json(s.webhook_urls.list().await?))
}
#[utoipa::path(get,path="/api/webhook-urls/{id}",params(("id"=String,Path)),responses((status=200,body=WebhookUrl),(status=404)))]
pub async fn get(State(s): State<AppState>, Path(id): Path<String>) -> AppResult<Json<WebhookUrl>> {
    Ok(Json(s.webhook_urls.get(&id).await?))
}
#[utoipa::path(post,path="/api/webhook-urls",request_body=WebhookUrlUpsertRequest,responses((status=201,body=WebhookUrl),(status=400)))]
pub async fn create(
    State(s): State<AppState>,
    Json(v): Json<WebhookUrlUpsertRequest>,
) -> AppResult<(StatusCode, Json<WebhookUrl>)> {
    Ok((StatusCode::CREATED, Json(s.webhook_urls.create(&v).await?)))
}
#[utoipa::path(put,path="/api/webhook-urls/{id}",params(("id"=String,Path)),request_body=WebhookUrlUpsertRequest,responses((status=200,body=WebhookUrl),(status=400),(status=404)))]
pub async fn update(
    State(s): State<AppState>,
    Path(id): Path<String>,
    Json(v): Json<WebhookUrlUpsertRequest>,
) -> AppResult<Json<WebhookUrl>> {
    Ok(Json(s.webhook_urls.update(&id, &v).await?))
}
#[utoipa::path(delete,path="/api/webhook-urls/{id}",params(("id"=String,Path)),responses((status=204),(status=404)))]
pub async fn delete(State(s): State<AppState>, Path(id): Path<String>) -> AppResult<StatusCode> {
    s.webhook_urls.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
