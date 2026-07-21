use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use kakeibo_app::router::expense_categories;
use serde_json::{Value, json};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;
async fn call(
    app: &axum::Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Option<Value>) {
    let mut b = Request::builder().method(method).uri(uri);
    let body = match body {
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
async fn expense_category_crud() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query("CREATE TABLE expense_categories(id TEXT PRIMARY KEY DEFAULT 'ec-1',created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,name TEXT NOT NULL,description TEXT)").execute(&pool).await.unwrap();
    let app = expense_categories::create(pool);
    let input = json!({"name":"Food","description":null});
    let (s, b) = call(&app, "POST", "/api/expense-categories", Some(input)).await;
    assert_eq!(s, StatusCode::CREATED, "{b:?}");
    let (s, b) = call(
        &app,
        "GET",
        "/api/expense-categories?page=1&per_page=50",
        None,
    )
    .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(b.unwrap()["items"][0]["name"], "Food");
    let (s, _) = call(&app, "DELETE", "/api/expense-categories/ec-1", None).await;
    assert_eq!(s, StatusCode::NO_CONTENT);
    let (s, _) = call(&app, "GET", "/api/expense-categories/ec-1", None).await;
    assert_eq!(s, StatusCode::NOT_FOUND)
}
