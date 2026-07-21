use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use kakeibo_app::router::expenses;
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
async fn expense_crud() {
    let p = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query("CREATE TABLE expense_categories(id TEXT PRIMARY KEY);CREATE TABLE payment_methods(id TEXT PRIMARY KEY);CREATE TABLE recurring_expenses(id TEXT PRIMARY KEY);CREATE TABLE expenses(id TEXT PRIMARY KEY DEFAULT 'e-1',created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,transaction_date TEXT NOT NULL,amount TEXT NOT NULL,category_id TEXT NOT NULL REFERENCES expense_categories(id),payment_method_id TEXT NOT NULL REFERENCES payment_methods(id),recurring_expense_id TEXT REFERENCES recurring_expenses(id),description TEXT);INSERT INTO expense_categories VALUES('ec');INSERT INTO payment_methods VALUES('pm')").execute(&p).await.unwrap();
    let app = expenses::create(p);
    let v = json!({"transaction_date":"2026-07-22","amount":"1200","category_id":"ec","payment_method_id":"pm","recurring_expense_id":null,"description":"Lunch"});
    let (s, b) = call(&app, "POST", "/api/expenses", Some(v)).await;
    assert_eq!(s, StatusCode::CREATED, "{b:?}");
    let (s, b) = call(&app, "GET", "/api/expenses", None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(b.unwrap()[0]["amount"], "1200");
    let (s, _) = call(&app, "DELETE", "/api/expenses/e-1", None).await;
    assert_eq!(s, StatusCode::NO_CONTENT)
}
