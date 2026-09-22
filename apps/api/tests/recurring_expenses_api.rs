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
    sqlx::query("CREATE TABLE expense_categories(id TEXT PRIMARY KEY);CREATE TABLE payment_methods(id TEXT PRIMARY KEY);CREATE TABLE recurring_expenses(id TEXT PRIMARY KEY DEFAULT 're-1',created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,name TEXT NOT NULL,amount TEXT NOT NULL,payment_day INTEGER NOT NULL,start_date TEXT NOT NULL,end_date TEXT,category_id TEXT NOT NULL REFERENCES expense_categories(id),payment_method_id TEXT NOT NULL REFERENCES payment_methods(id),is_active INTEGER NOT NULL,is_variable INTEGER NOT NULL DEFAULT 0,description TEXT,foreign_amount TEXT,currency_code TEXT,exchange_rate TEXT);CREATE TABLE expenses(id TEXT PRIMARY KEY,updated_at TEXT NOT NULL DEFAULT current_timestamp,transaction_date TEXT NOT NULL,amount TEXT NOT NULL,category_id TEXT NOT NULL,payment_method_id TEXT NOT NULL,recurring_expense_id TEXT,description TEXT);INSERT INTO expense_categories VALUES('ec');INSERT INTO payment_methods VALUES('pm')").execute(&p).await.unwrap();
    let app = recurring_expenses::create(p);
    let v = json!({"name":"Rent","amount":"80000","payment_day":27,"start_date":"2026-01-01","end_date":null,"category_id":"ec","payment_method_id":"pm","is_active":true,"is_variable":true,"description":null});
    let (s, b) = call(&app, "POST", "/api/recurring-expenses", Some(v)).await;
    assert_eq!(s, StatusCode::CREATED, "{b:?}");
    let (s, b) = call(&app, "GET", "/api/recurring-expenses", None).await;
    assert_eq!(s, StatusCode::OK);
    let b = b.unwrap();
    assert_eq!(b[0]["name"], "Rent");
    assert_eq!(b[0]["is_variable"], true);
    assert_eq!(b[0]["foreign_amount"], Value::Null);
    let (s, _) = call(&app, "DELETE", "/api/recurring-expenses/re-1", None).await;
    assert_eq!(s, StatusCode::NO_CONTENT)
}

#[tokio::test]
async fn recurring_expense_supports_foreign_currency() {
    let p = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query("CREATE TABLE expense_categories(id TEXT PRIMARY KEY);CREATE TABLE payment_methods(id TEXT PRIMARY KEY);CREATE TABLE recurring_expenses(id TEXT PRIMARY KEY DEFAULT 're-1',created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,name TEXT NOT NULL,amount TEXT NOT NULL,payment_day INTEGER NOT NULL,start_date TEXT NOT NULL,end_date TEXT,category_id TEXT NOT NULL REFERENCES expense_categories(id),payment_method_id TEXT NOT NULL REFERENCES payment_methods(id),is_active INTEGER NOT NULL,is_variable INTEGER NOT NULL DEFAULT 0,description TEXT,foreign_amount TEXT,currency_code TEXT,exchange_rate TEXT);CREATE TABLE expenses(id TEXT PRIMARY KEY,updated_at TEXT NOT NULL DEFAULT current_timestamp,transaction_date TEXT NOT NULL,amount TEXT NOT NULL,category_id TEXT NOT NULL,payment_method_id TEXT NOT NULL,recurring_expense_id TEXT,description TEXT);INSERT INTO expense_categories VALUES('ec');INSERT INTO payment_methods VALUES('pm')").execute(&p).await.unwrap();
    let app = recurring_expenses::create(p);
    let v = json!({"name":"Cloud","amount":"1573","payment_day":1,"start_date":"2026-01-01","end_date":null,"category_id":"ec","payment_method_id":"pm","is_active":true,"is_variable":false,"description":null,"foreign_amount":"10","currency_code":"USD","exchange_rate":"157.3"});
    let (s, b) = call(&app, "POST", "/api/recurring-expenses", Some(v)).await;
    assert_eq!(s, StatusCode::CREATED, "{b:?}");
    assert_eq!(b.unwrap()["currency_code"], "USD");
}

#[tokio::test]
async fn update_can_sync_generated_transactions_from_current_month() {
    let p = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query("CREATE TABLE expense_categories(id TEXT PRIMARY KEY);CREATE TABLE payment_methods(id TEXT PRIMARY KEY);CREATE TABLE recurring_expenses(id TEXT PRIMARY KEY DEFAULT 're-1',created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,name TEXT NOT NULL,amount TEXT NOT NULL,payment_day INTEGER NOT NULL,start_date TEXT NOT NULL,end_date TEXT,category_id TEXT NOT NULL REFERENCES expense_categories(id),payment_method_id TEXT NOT NULL REFERENCES payment_methods(id),is_active INTEGER NOT NULL,is_variable INTEGER NOT NULL DEFAULT 0,description TEXT,foreign_amount TEXT,currency_code TEXT,exchange_rate TEXT);CREATE TABLE expenses(id TEXT PRIMARY KEY,updated_at TEXT NOT NULL DEFAULT current_timestamp,transaction_date TEXT NOT NULL,amount TEXT NOT NULL,category_id TEXT NOT NULL,payment_method_id TEXT NOT NULL,recurring_expense_id TEXT,description TEXT);INSERT INTO expense_categories VALUES('ec');INSERT INTO payment_methods VALUES('pm')").execute(&p).await.unwrap();
    let app = recurring_expenses::create(p.clone());
    let base = json!({"name":"Rent","amount":"80000","payment_day":27,"start_date":"2026-01-01","end_date":null,"category_id":"ec","payment_method_id":"pm","is_active":true,"is_variable":false,"description":null});
    assert_eq!(
        call(&app, "POST", "/api/recurring-expenses", Some(base.clone()))
            .await
            .0,
        StatusCode::CREATED
    );
    sqlx::query("INSERT INTO expenses(id, transaction_date, amount, category_id, payment_method_id, recurring_expense_id) VALUES('past', date('now','localtime','start of month','-1 day'), '80000', 'ec', 'pm', 're-1'), ('future', date('now','localtime','start of month'), '80000', 'ec', 'pm', 're-1')").execute(&p).await.unwrap();
    let mut update = base;
    update["amount"] = json!("90000");
    update["sync_future_transactions"] = json!(true);
    assert_eq!(
        call(&app, "PUT", "/api/recurring-expenses/re-1", Some(update))
            .await
            .0,
        StatusCode::OK
    );
    let rows: Vec<(String, String)> = sqlx::query_as("SELECT id, amount FROM expenses ORDER BY id")
        .fetch_all(&p)
        .await
        .unwrap();
    assert_eq!(
        rows,
        vec![
            ("future".into(), "90000".into()),
            ("past".into(), "80000".into())
        ]
    );
}
