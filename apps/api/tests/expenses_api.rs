use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use kakeibo_app::{repository::expenses::ExpenseRepository, router::expenses};
use serde_json::{Value, json};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
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
const SCHEMA: &str = concat!(
    "CREATE TABLE expense_categories(id TEXT PRIMARY KEY);",
    "CREATE TABLE payment_methods(id TEXT PRIMARY KEY);",
    "CREATE TABLE recurring_expenses(id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))||'-'||hex(randomblob(2))||'-'||'4'||substr(hex(randomblob(2)),2)||'-'||substr('AB89',1+(abs(random())%4),1)||substr(hex(randomblob(2)),2)||'-'||hex(randomblob(6)))),created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,name TEXT NOT NULL,amount TEXT NOT NULL,payment_day INTEGER NOT NULL,start_date TEXT NOT NULL,end_date TEXT,category_id TEXT NOT NULL REFERENCES expense_categories(id),payment_method_id TEXT NOT NULL REFERENCES payment_methods(id),is_active INTEGER NOT NULL,is_variable INTEGER NOT NULL DEFAULT 0,description TEXT);",
    "CREATE TABLE expenses(id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))||'-'||hex(randomblob(2))||'-'||'4'||substr(hex(randomblob(2)),2)||'-'||substr('AB89',1+(abs(random())%4),1)||substr(hex(randomblob(2)),2)||'-'||hex(randomblob(6)))),created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,transaction_date TEXT NOT NULL,amount TEXT NOT NULL,category_id TEXT NOT NULL REFERENCES expense_categories(id),payment_method_id TEXT NOT NULL REFERENCES payment_methods(id),recurring_expense_id TEXT REFERENCES recurring_expenses(id),description TEXT);",
    "INSERT INTO expense_categories VALUES('ec');",
    "INSERT INTO payment_methods VALUES('pm');",
    "CREATE TABLE webhook_urls(id TEXT PRIMARY KEY,url TEXT NOT NULL,description TEXT,is_active INTEGER NOT NULL DEFAULT 1);",
);
async fn setup_pool() -> SqlitePool {
    let p = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query(SCHEMA).execute(&p).await.unwrap();
    p
}
async fn current_month(p: &SqlitePool) -> String {
    let (month,): (String,) = sqlx::query_as("SELECT strftime('%Y-%m','now','localtime')")
        .fetch_one(p)
        .await
        .unwrap();
    month
}
#[tokio::test]
async fn expense_crud() {
    let p = setup_pool().await;
    let app = expenses::create(p);
    let v = json!({"transaction_date":"2026-07-22","amount":"1200","category_id":"ec","payment_method_id":"pm","recurring_expense_id":null,"description":"Lunch"});
    let (s, b) = call(&app, "POST", "/api/expenses", Some(v)).await;
    assert_eq!(s, StatusCode::CREATED, "{b:?}");
    let id = b.unwrap()["id"].as_str().unwrap().to_string();
    let (s, b) = call(&app, "GET", "/api/expenses", None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(b.unwrap()[0]["amount"], "1200");
    let (s, _) = call(&app, "DELETE", &format!("/api/expenses/{id}"), None).await;
    assert_eq!(s, StatusCode::NO_CONTENT)
}
#[tokio::test]
async fn list_posts_recurring_expenses_for_current_month() {
    let p = setup_pool().await;
    let month = current_month(&p).await;
    sqlx::query("INSERT INTO recurring_expenses(name,amount,payment_day,start_date,end_date,category_id,payment_method_id,is_active,description) VALUES('家賃','98000',15,?1,NULL,'ec','pm',1,NULL),('駐車場','5000',15,?1,NULL,'ec','pm',0,NULL)").bind(format!("{month}-01")).execute(&p).await.unwrap();
    let app = expenses::create(p);

    let (s, b) = call(&app, "GET", "/api/expenses", None).await;
    assert_eq!(s, StatusCode::OK);
    let b = b.unwrap();
    let items = b.as_array().unwrap();
    // 有効な定期支出のみ当月15日付で計上される（無効なものは対象外）
    assert_eq!(items.len(), 1, "{items:?}");
    assert_eq!(items[0]["transaction_date"], json!(format!("{month}-15")));
    assert_eq!(items[0]["amount"], json!("98000"));
    assert_ne!(items[0]["recurring_expense_id"], json!(null));

    // 再度呼び出しても二重計上されない
    let (s, b) = call(&app, "GET", "/api/expenses", None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(b.unwrap().as_array().unwrap().len(), 1);
}
#[tokio::test]
async fn insert_recurring_for_month_clamps_payment_day_to_month_end() {
    let p = setup_pool().await;
    sqlx::query("INSERT INTO recurring_expenses(name,amount,payment_day,start_date,end_date,category_id,payment_method_id,is_active,description) VALUES('家賃','98000',31,'2026-01-01',NULL,'ec','pm',1,NULL)").execute(&p).await.unwrap();
    let repo = ExpenseRepository::new(p.clone());

    // 2026年2月は28日までしかないため、28日付で計上される
    assert_eq!(repo.insert_recurring_for_month("2026-02").await.unwrap(), 1);
    let (date,): (String,) = sqlx::query_as("SELECT transaction_date FROM expenses")
        .fetch_one(&p)
        .await
        .unwrap();
    assert_eq!(date, "2026-02-28");
    // 同月の再実行は冪等、翌月は新たに計上される
    assert_eq!(repo.insert_recurring_for_month("2026-02").await.unwrap(), 0);
    assert_eq!(repo.insert_recurring_for_month("2026-03").await.unwrap(), 1);
}
#[tokio::test]
async fn insert_recurring_for_month_respects_active_period() {
    let p = setup_pool().await;
    sqlx::query("INSERT INTO recurring_expenses(name,amount,payment_day,start_date,end_date,category_id,payment_method_id,is_active,description) VALUES('家賃','98000',15,'2026-03-01',NULL,'ec','pm',1,NULL),('駐車場','5000',15,'2025-01-01','2026-01-31','ec','pm',1,NULL)").execute(&p).await.unwrap();
    let repo = ExpenseRepository::new(p.clone());

    // 開始前・終了済みの定期支出は計上されない
    assert_eq!(repo.insert_recurring_for_month("2026-02").await.unwrap(), 0);
    // 開始月には計上される
    assert_eq!(repo.insert_recurring_for_month("2026-03").await.unwrap(), 1);
}
#[tokio::test]
async fn insert_recurring_for_month_skips_variable_expenses() {
    let p = setup_pool().await;
    // 準固定費（is_variable=1）は金額が月ごとに変動するため自動計上の対象外
    sqlx::query("INSERT INTO recurring_expenses(name,amount,payment_day,start_date,end_date,category_id,payment_method_id,is_active,is_variable,description) VALUES('電気代','8000',15,'2026-01-01',NULL,'ec','pm',1,1,NULL)").execute(&p).await.unwrap();
    let repo = ExpenseRepository::new(p.clone());

    assert_eq!(repo.insert_recurring_for_month("2026-02").await.unwrap(), 0);
}
#[tokio::test]
async fn create_succeeds_even_when_webhook_notification_fails() {
    let p = setup_pool().await;
    // 接続不能な通知先URLが登録されていても、支出の登録自体は成功する
    sqlx::query(
        "INSERT INTO webhook_urls(id,url,is_active) VALUES('wu-1','http://127.0.0.1:1/hook',1)",
    )
    .execute(&p)
    .await
    .unwrap();
    let app = expenses::create(p);
    let v = json!({"transaction_date":"2026-07-22","amount":"1200","category_id":"ec","payment_method_id":"pm","recurring_expense_id":null,"description":null});
    let (s, b) = call(&app, "POST", "/api/expenses", Some(v)).await;
    assert_eq!(s, StatusCode::CREATED, "{b:?}");
}
