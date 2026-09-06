use axum::{
    body::{Body, to_bytes},
    http::{HeaderMap, Request, StatusCode},
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
async fn get(app: &axum::Router, uri: &str) -> (StatusCode, HeaderMap, Vec<u8>) {
    let response = app
        .clone()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap()
        .to_vec();
    (status, headers, body)
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
const EXPENSE_SCHEMA: &str = "CREATE TABLE expense_categories(id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,name TEXT NOT NULL,description TEXT,parent_category_id TEXT REFERENCES expense_categories(id));CREATE TABLE payment_methods(id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,name TEXT NOT NULL,description TEXT,initial_balance TEXT);CREATE TABLE recurring_expenses(id TEXT PRIMARY KEY);CREATE TABLE expenses(id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,transaction_date TEXT NOT NULL,amount TEXT NOT NULL,category_id TEXT NOT NULL REFERENCES expense_categories(id),payment_method_id TEXT NOT NULL REFERENCES payment_methods(id),recurring_expense_id TEXT REFERENCES recurring_expenses(id),description TEXT);INSERT INTO expense_categories(name,description) VALUES('食費',NULL);INSERT INTO payment_methods(name,description) VALUES('現金',NULL)";
const INCOME_SCHEMA: &str = "CREATE TABLE income_categories(id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,name TEXT NOT NULL,description TEXT,parent_category_id TEXT REFERENCES income_categories(id));CREATE TABLE incomes(id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,category_id TEXT NOT NULL REFERENCES income_categories(id),transaction_date TEXT NOT NULL,amount TEXT NOT NULL,payment_method_id TEXT,recurring_income_id TEXT,description TEXT);INSERT INTO income_categories(name,description) VALUES('給与',NULL)";

#[tokio::test]
async fn download_expense_import_sample() {
    let app = import::create(pool(EXPENSE_SCHEMA).await);
    let (status, headers, body) = get(&app, "/api/import/expenses/sample").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers["content-type"], "text/csv; charset=utf-8");
    assert_eq!(
        headers["content-disposition"],
        "attachment; filename=\"expense_import_sample.csv\""
    );
    let csv = String::from_utf8(body).unwrap();
    assert!(csv.starts_with('\u{feff}'));
    assert_eq!(
        csv,
        "\u{feff}日付,金額,カテゴリ,支払方法,メモ\n2026-01-15,1200,食費,現金,昼食\n"
    );
}

#[tokio::test]
async fn download_income_import_sample() {
    let app = import::create(pool(INCOME_SCHEMA).await);
    let (status, headers, body) = get(&app, "/api/import/incomes/sample").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers["content-type"], "text/csv; charset=utf-8");
    assert_eq!(
        headers["content-disposition"],
        "attachment; filename=\"income_import_sample.csv\""
    );
    let csv = String::from_utf8(body).unwrap();
    assert!(csv.starts_with('\u{feff}'));
    assert_eq!(
        csv,
        "\u{feff}日付,金額,カテゴリ,メモ\n2026-01-15,300000,給与,1月分\n"
    );
}

#[tokio::test]
async fn downloaded_samples_can_be_imported() {
    let expense_pool = pool(EXPENSE_SCHEMA).await;
    let expense_app = import::create(expense_pool.clone());
    let (_, _, expense_csv) = get(&expense_app, "/api/import/expenses/sample").await;
    let expense_csv = String::from_utf8(expense_csv).unwrap();
    let (status, body) = call(&expense_app, "/api/import/expenses", &expense_csv).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["imported"], 1);
    assert_eq!(
        count(&expense_pool, "SELECT COUNT(*) FROM expenses").await,
        1
    );

    let income_pool = pool(INCOME_SCHEMA).await;
    let income_app = import::create(income_pool.clone());
    let (_, _, income_csv) = get(&income_app, "/api/import/incomes/sample").await;
    let income_csv = String::from_utf8(income_csv).unwrap();
    let (status, body) = call(&income_app, "/api/import/incomes", &income_csv).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["imported"], 1);
    assert_eq!(count(&income_pool, "SELECT COUNT(*) FROM incomes").await, 1);
}
#[tokio::test]
async fn import_expenses_csv() {
    let p = pool(EXPENSE_SCHEMA).await;
    sqlx::query(
        "ALTER TABLE expense_categories ADD COLUMN display_order INTEGER NOT NULL DEFAULT 0",
    )
    .execute(&p)
    .await
    .unwrap();
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
            "SELECT display_order FROM expense_categories WHERE name='医療費'"
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
    let p = pool(INCOME_SCHEMA).await;
    sqlx::query(
        "ALTER TABLE income_categories ADD COLUMN display_order INTEGER NOT NULL DEFAULT 0",
    )
    .execute(&p)
    .await
    .unwrap();
    let app = import::create(p.clone());
    let csv = "日付,金額,カテゴリ,メモ\n2026-07-25,300000,給与,7月分\n2026-07-10,20000,賞与,\n";
    let (s, b) = call(&app, "/api/import/incomes", csv).await;
    assert_eq!(s, StatusCode::CREATED, "{b}");
    assert_eq!(b["imported"], 2);
    assert_eq!(b["created_categories"], json!(["賞与"]));
    assert_eq!(b["created_payment_methods"], json!([]));
    assert_eq!(
        count(
            &p,
            "SELECT display_order FROM income_categories WHERE name='賞与'"
        )
        .await,
        1
    );
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
