use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use kakeibo_app::router::webhook_urls;
use serde_json::{Value, json};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;
async fn call(
    app: &axum::Router,
    m: &str,
    u: &str,
    v: Option<Value>,
) -> (StatusCode, Option<Value>) {
    let mut b = Request::builder().method(m).uri(u);
    let body = match v {
        Some(v) => {
            b = b.header("content-type", "application/json");
            Body::from(v.to_string())
        }
        None => Body::empty(),
    };
    let r = app.clone().oneshot(b.body(body).unwrap()).await.unwrap();
    let s = r.status();
    let bytes = to_bytes(r.into_body(), usize::MAX).await.unwrap();
    (
        s,
        (!bytes.is_empty()).then(|| serde_json::from_slice(&bytes).unwrap()),
    )
}
async fn setup_pool() -> sqlx::SqlitePool {
    let p = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query("CREATE TABLE webhook_urls(id TEXT PRIMARY KEY DEFAULT 'wu-1',created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,url TEXT NOT NULL,description TEXT,is_active INTEGER NOT NULL DEFAULT 1)").execute(&p).await.unwrap();
    p
}
#[tokio::test]
async fn webhook_url_crud() {
    let p = setup_pool().await;
    let app = webhook_urls::create(p);
    let v = json!({"url":"https://example.com/hook","description":"通知先1","is_active":true});
    let (s, b) = call(&app, "POST", "/api/webhook-urls", Some(v)).await;
    assert_eq!(s, StatusCode::CREATED, "{b:?}");
    assert_eq!(b.as_ref().unwrap()["is_active"], true);

    let (s, b) = call(&app, "GET", "/api/webhook-urls", None).await;
    assert_eq!(s, StatusCode::OK);
    let b = b.unwrap();
    assert_eq!(b[0]["url"], "https://example.com/hook");
    assert_eq!(b[0]["is_active"], true);

    let v = json!({"url":"https://example.com/hook2","description":null,"is_active":false});
    let (s, b) = call(&app, "PUT", "/api/webhook-urls/wu-1", Some(v)).await;
    assert_eq!(s, StatusCode::OK, "{b:?}");
    assert_eq!(b.as_ref().unwrap()["is_active"], false);

    let (s, _) = call(&app, "DELETE", "/api/webhook-urls/wu-1", None).await;
    assert_eq!(s, StatusCode::NO_CONTENT);
    let (s, _) = call(&app, "GET", "/api/webhook-urls/wu-1", None).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
}
#[tokio::test]
async fn rejects_url_without_http_scheme() {
    let p = setup_pool().await;
    let app = webhook_urls::create(p);
    let v = json!({"url":"example.com","description":null,"is_active":true});
    let (s, _) = call(&app, "POST", "/api/webhook-urls", Some(v)).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
}
