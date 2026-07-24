use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use kakeibo_app::router::import;
use serde_json::{Value, json};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;
async fn call(app: &axum::Router, u: &str, csv: &str) -> (StatusCode, Value) {
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(u)
                .header("content-type", "text/csv")
                .body(Body::from(csv.to_owned()))
                .unwrap(),
        )
        .await
        .unwrap();
    let s = r.status();
    let bytes = to_bytes(r.into_body(), usize::MAX).await.unwrap();
    (s, serde_json::from_slice(&bytes).unwrap())
}
async fn pool(schema: &'static str) -> sqlx::SqlitePool {
    let p = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query(schema).execute(&p).await.unwrap();
    p
}
async fn count(p: &sqlx::SqlitePool, sql: &'static str) -> i64 {
    sqlx::query_scalar::<_, i64>(sql)
        .fetch_one(p)
        .await
        .unwrap()
}
const EXPENSE_SCHEMA: &str = "CREATE TABLE expense_categories(id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,name TEXT NOT NULL,description TEXT);CREATE TABLE payment_methods(id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,name TEXT NOT NULL,description TEXT);CREATE TABLE recurring_expenses(id TEXT PRIMARY KEY);CREATE TABLE expenses(id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,transaction_date TEXT NOT NULL,amount TEXT NOT NULL,category_id TEXT NOT NULL REFERENCES expense_categories(id),payment_method_id TEXT NOT NULL REFERENCES payment_methods(id),recurring_expense_id TEXT REFERENCES recurring_expenses(id),description TEXT);INSERT INTO expense_categories(name,description) VALUES('食費',NULL);INSERT INTO payment_methods(name,description) VALUES('現金',NULL)";
#[tokio::test]
async fn import_expenses_csv() {
    let p = pool(EXPENSE_SCHEMA).await;
    let app = import::create(p.clone());
    let csv = "\u{feff}日付,金額,カテゴリ,支払方法,メモ\r\n2026-07-22,1200,食費,現金,\"ランチ, 友人と\"\r\n2026-07-23,800,医療費,電子マネー,\r\n";
    let (s, b) = call(&app, "/api/import/expenses", csv).await;
    assert_eq!(s, StatusCode::CREATED, "{b}");
    assert_eq!(b["imported"], 2);
    assert_eq!(b["created_categories"], json!(["医療費"]));
    assert_eq!(b["created_payment_methods"], json!(["電子マネー"]));
    assert_eq!(count(&p, "SELECT COUNT(*) FROM expenses").await, 2);
    assert_eq!(
        count(
            &p,
            "SELECT COUNT(*) FROM expense_categories WHERE name='医療費'"
        )
        .await,
        1
    );
    assert_eq!(
        count(
            &p,
            "SELECT COUNT(*) FROM payment_methods WHERE name='電子マネー'"
        )
        .await,
        1
    );
    let description: Option<String> =
        sqlx::query_scalar("SELECT description FROM expenses WHERE transaction_date='2026-07-22'")
            .fetch_one(&p)
            .await
            .unwrap();
    assert_eq!(description.as_deref(), Some("ランチ, 友人と"));
    let description: Option<String> =
        sqlx::query_scalar("SELECT description FROM expenses WHERE transaction_date='2026-07-23'")
            .fetch_one(&p)
            .await
            .unwrap();
    assert_eq!(description, None);
}
#[tokio::test]
async fn import_incomes_csv() {
    let p = pool("CREATE TABLE income_categories(id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,name TEXT NOT NULL,description TEXT);CREATE TABLE incomes(id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,category_id TEXT NOT NULL REFERENCES income_categories(id),transaction_date TEXT NOT NULL,amount TEXT NOT NULL,description TEXT);INSERT INTO income_categories(name,description) VALUES('給与',NULL)").await;
    let app = import::create(p.clone());
    let csv = "日付,金額,カテゴリ,メモ\n2026-07-25,300000,給与,7月分\n2026-07-10,20000,賞与,\n";
    let (s, b) = call(&app, "/api/import/incomes", csv).await;
    assert_eq!(s, StatusCode::CREATED, "{b}");
    assert_eq!(b["imported"], 2);
    assert_eq!(b["created_categories"], json!(["賞与"]));
    assert_eq!(b["created_payment_methods"], json!([]));
    assert_eq!(count(&p, "SELECT COUNT(*) FROM incomes").await, 2);
    assert_eq!(
        count(
            &p,
            "SELECT COUNT(*) FROM income_categories WHERE name='賞与'"
        )
        .await,
        1
    );
}
#[tokio::test]
async fn import_rejects_invalid_rows_without_partial_writes() {
    let p = pool(EXPENSE_SCHEMA).await;
    let app = import::create(p.clone());
    let csv = "日付,金額,カテゴリ,支払方法,メモ\n2026/07/22,1200,食費,現金,\n2026-07-23,abc,食費,現金,\n2026-07-24,500,新規カテゴリ,現金,\n";
    let (s, b) = call(&app, "/api/import/expenses", csv).await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "{b}");
    let message = b["message"].as_str().unwrap();
    assert!(
        message.contains("2行目") && message.contains("3行目"),
        "{message}"
    );
    assert_eq!(count(&p, "SELECT COUNT(*) FROM expenses").await, 0);
    assert_eq!(
        count(
            &p,
            "SELECT COUNT(*) FROM expense_categories WHERE name='新規カテゴリ'"
        )
        .await,
        0
    );
}
#[tokio::test]
async fn import_rejects_wrong_headers() {
    let p = pool(EXPENSE_SCHEMA).await;
    let app = import::create(p.clone());
    let (s, b) = call(
        &app,
        "/api/import/incomes",
        "date,amount\n2026-07-25,300000\n",
    )
    .await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "{b}");
    assert!(b["message"].as_str().unwrap().contains("ヘッダー"), "{b}");
    let (s, b) = call(&app, "/api/import/expenses", "").await;
    assert_eq!(s, StatusCode::BAD_REQUEST, "{b}");
    assert!(b["message"].as_str().unwrap().contains("空です"), "{b}");
}
