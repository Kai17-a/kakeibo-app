use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use kakeibo_app::router::recurring_expenses;
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
async fn recurring_expense_crud() {
    let p = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query("CREATE TABLE expense_categories(id TEXT PRIMARY KEY);CREATE TABLE payment_methods(id TEXT PRIMARY KEY);CREATE TABLE recurring_expenses(id TEXT PRIMARY KEY DEFAULT 're-1',created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,name TEXT NOT NULL,amount TEXT NOT NULL,payment_day INTEGER NOT NULL,start_date TEXT NOT NULL,end_date TEXT,category_id TEXT NOT NULL REFERENCES expense_categories(id),payment_method_id TEXT NOT NULL REFERENCES payment_methods(id),is_active INTEGER NOT NULL,description TEXT);INSERT INTO expense_categories VALUES('ec');INSERT INTO payment_methods VALUES('pm')").execute(&p).await.unwrap();
    let app = recurring_expenses::create(p);
    let v = json!({"name":"Rent","amount":"80000","payment_day":27,"start_date":"2026-01-01","end_date":null,"category_id":"ec","payment_method_id":"pm","is_active":true,"description":null});
    let (s, b) = call(&app, "POST", "/api/recurring-expenses", Some(v)).await;
    assert_eq!(s, StatusCode::CREATED, "{b:?}");
    let (s, b) = call(&app, "GET", "/api/recurring-expenses", None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(b.unwrap()[0]["name"], "Rent");
    let (s, _) = call(&app, "DELETE", "/api/recurring-expenses/re-1", None).await;
    assert_eq!(s, StatusCode::NO_CONTENT)
}
