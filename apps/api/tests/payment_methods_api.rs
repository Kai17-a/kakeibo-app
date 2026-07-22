use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use kakeibo_app::router::payment_methods;
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
#[tokio::test]
async fn payment_method_crud() {
    let p = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query("CREATE TABLE payment_methods(id TEXT PRIMARY KEY DEFAULT 'pm-1',created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,name TEXT NOT NULL,description TEXT)").execute(&p).await.unwrap();
    let app = payment_methods::create(p);
    let (s, _) = call(
        &app,
        "POST",
        "/api/payment-methods",
        Some(json!({"name":"Cash","description":null})),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED);
    let (s, b) = call(&app, "GET", "/api/payment-methods", None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(b.unwrap()["items"][0]["name"], "Cash");
    let (s, _) = call(&app, "DELETE", "/api/payment-methods/pm-1", None).await;
    assert_eq!(s, StatusCode::NO_CONTENT)
}
