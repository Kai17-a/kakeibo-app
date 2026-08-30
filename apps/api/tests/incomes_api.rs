use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use kakeibo_app::{repository::incomes::IncomeRepository, router::incomes};
use serde_json::{Value, json};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::{
    io::Write,
    sync::{Arc, Mutex},
    time::Duration,
};
use tower::ServiceExt;

async fn pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query(
        "CREATE TABLE income_categories (id TEXT PRIMARY KEY); \
         CREATE TABLE recurring_incomes (id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))), \
         created_at TEXT NOT NULL DEFAULT current_timestamp, updated_at TEXT NOT NULL DEFAULT current_timestamp, \
         name TEXT NOT NULL, amount TEXT NOT NULL, payment_day INTEGER NOT NULL, start_date TEXT NOT NULL, \
         end_date TEXT, category_id TEXT NOT NULL REFERENCES income_categories(id), is_active INTEGER NOT NULL, \
         is_variable INTEGER NOT NULL DEFAULT 0, description TEXT); \
         CREATE TABLE incomes (id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))), \
         created_at TEXT NOT NULL DEFAULT current_timestamp, \
         updated_at TEXT NOT NULL DEFAULT current_timestamp, category_id TEXT NOT NULL \
         REFERENCES income_categories(id), transaction_date TEXT NOT NULL, amount TEXT NOT NULL, \
         recurring_income_id TEXT REFERENCES recurring_incomes(id), description TEXT)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO income_categories (id) VALUES ('salary')")
        .execute(&pool)
        .await
        .unwrap();
    pool
}

async fn app() -> Router {
    incomes::create(pool().await)
}

async fn app_with_incomes() -> Router {
    let pool = pool().await;
    sqlx::query("INSERT INTO income_categories (id) VALUES ('freelance')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO incomes (id, category_id, transaction_date, amount, description) VALUES \
         ('income-a', 'salary', '2026-07-01', '100', NULL), \
         ('income-b', 'salary', '2026-07-15', '300', NULL), \
         ('income-c', 'freelance', '2026-07-10', '200', NULL), \
         ('income-d', 'salary', '2026-06-30', '400', NULL)",
    )
    .execute(&pool)
    .await
    .unwrap();
    incomes::create(pool)
}

async fn call(
    app: &Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Option<Value>) {
    let mut builder = Request::builder().method(method).uri(uri);
    let request_body = match body {
        Some(body) => {
            builder = builder.header("content-type", "application/json");
            Body::from(body.to_string())
        }
        None => Body::empty(),
    };
    let response = app
        .clone()
        .oneshot(builder.body(request_body).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = (!bytes.is_empty()).then(|| serde_json::from_slice(&bytes).unwrap());
    (status, body)
}

#[tokio::test]
async fn income_crud_lifecycle() {
    let app = app().await;
    let input = json!({
        "category_id": "salary",
        "transaction_date": "2026-07-19",
        "amount": "300000",
        "description": "July salary"
    });

    let (status, created) = call(&app, "POST", "/api/incomes", Some(input.clone())).await;
    assert_eq!(status, StatusCode::CREATED, "{created:?}");
    assert_eq!(created.as_ref().unwrap()["amount"], "300000");
    let id = created.as_ref().unwrap()["id"].as_str().unwrap().to_owned();

    let (status, list) = call(&app, "GET", "/api/incomes", None).await;
    assert_eq!(status, StatusCode::OK, "{list:?}");
    let list = list.unwrap();
    assert_eq!(list["items"].as_array().unwrap().len(), 1);
    assert_eq!(list["pagination"]["page"], 1);
    assert_eq!(list["pagination"]["per_page"], 50);

    let updated = json!({
        "category_id": "salary",
        "transaction_date": "2026-07-19",
        "amount": "310000",
        "description": "July salary"
    });
    let (status, body) = call(&app, "PUT", &format!("/api/incomes/{id}"), Some(updated)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.unwrap()["amount"], "310000");

    let (status, _) = call(&app, "DELETE", &format!("/api/incomes/{id}"), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = call(&app, "GET", &format!("/api/incomes/{id}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn filters_sorts_and_paginates_incomes() {
    let app = app_with_incomes().await;
    let uri = "/api/incomes?category_id=salary&date_from=2026-07-01&date_to=2026-07-31&sort_by=amount&sort_order=asc&page=1&per_page=1";

    let (status, first_page) = call(&app, "GET", uri, None).await;
    assert_eq!(status, StatusCode::OK);
    let first_page = first_page.unwrap();
    assert_eq!(first_page["items"][0]["id"], "income-a");
    assert_eq!(first_page["pagination"]["page"], 1);
    assert_eq!(first_page["pagination"]["per_page"], 1);

    let second_page_uri = "/api/incomes?category_id=salary&date_from=2026-07-01&date_to=2026-07-31&sort_by=amount&sort_order=asc&page=2&per_page=1";
    let (status, second_page) = call(&app, "GET", second_page_uri, None).await;
    assert_eq!(status, StatusCode::OK);
    let second_page = second_page.unwrap();
    assert_eq!(second_page["items"][0]["id"], "income-b");
    assert_eq!(second_page["pagination"]["page"], 2);
    assert_eq!(second_page["pagination"]["per_page"], 1);

    let (status, by_id) = call(&app, "GET", "/api/incomes?id=income-c", None).await;
    assert_eq!(status, StatusCode::OK);
    let by_id = by_id.unwrap();
    assert_eq!(by_id["items"][0]["category_id"], "freelance");
}

#[tokio::test]
async fn rejects_invalid_query_parameters() {
    let app = app_with_incomes().await;
    let invalid_queries = [
        "unknown=value",
        "sort_by=unknown",
        "sort_order=unknown",
        "date_from=2026-02-30",
        "date_from=2026-07-20&date_to=2026-07-01",
        "page=0",
        "per_page=101",
    ];

    for query in invalid_queries {
        let uri = format!("/api/incomes?{query}");
        let (status, body) = call(&app, "GET", &uri, None).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{query}: {body:?}");
    }
}

#[tokio::test]
async fn list_posts_only_eligible_recurring_incomes_and_is_idempotent() {
    let pool = pool().await;
    let month: String = sqlx::query_scalar("SELECT strftime('%Y-%m','now','localtime')")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO recurring_incomes(id,name,amount,payment_day,start_date,end_date,category_id,is_active,is_variable,description) VALUES ('salary-rule','給与','300000',25,?1,NULL,'salary',1,0,'給与'), ('inactive','無効','1',1,?1,NULL,'salary',0,0,NULL), ('variable','歩合','50000',20,?1,NULL,'salary',1,1,NULL)")
        .bind(format!("{month}-01")).execute(&pool).await.unwrap();
    let app = incomes::create(pool);

    let (_, first) = call(&app, "GET", "/api/incomes", None).await;
    let items = first.unwrap()["items"].as_array().unwrap().clone();
    assert_eq!(items.len(), 1, "{items:?}");
    assert_eq!(items[0]["recurring_income_id"], "salary-rule");
    let (_, second) = call(&app, "GET", "/api/incomes", None).await;
    assert_eq!(second.unwrap()["items"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn invalid_list_query_does_not_post_recurring_income() {
    let pool = pool().await;
    sqlx::query("INSERT INTO recurring_incomes(id,name,amount,payment_day,start_date,category_id,is_active,is_variable) VALUES ('salary-rule','給与','300000',25,'2020-01-01','salary',1,0)")
        .execute(&pool).await.unwrap();
    let app = incomes::create(pool.clone());
    let (status, _) = call(&app, "GET", "/api/incomes?page=0", None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM incomes")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn repository_clamps_month_end_is_idempotent_and_posts_next_month() {
    let pool = pool().await;
    sqlx::query("INSERT INTO recurring_incomes(id,name,amount,payment_day,start_date,category_id,is_active,is_variable) VALUES ('salary-rule','給与','300000',31,'2026-01-01','salary',1,0)")
        .execute(&pool).await.unwrap();
    let repo = IncomeRepository::new(pool.clone());
    assert_eq!(repo.insert_recurring_for_month("2026-02").await.unwrap(), 1);
    let date: String = sqlx::query_scalar("SELECT transaction_date FROM incomes")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(date, "2026-02-28");
    assert_eq!(repo.insert_recurring_for_month("2026-02").await.unwrap(), 0);
    assert_eq!(repo.insert_recurring_for_month("2026-03").await.unwrap(), 1);
}

#[tokio::test]
async fn repository_respects_active_period_and_skips_variable_income() {
    let pool = pool().await;
    sqlx::query("INSERT INTO recurring_incomes(id,name,amount,payment_day,start_date,end_date,category_id,is_active,is_variable) VALUES ('future','将来','1',1,'2026-03-01',NULL,'salary',1,0), ('ended','終了','1',1,'2025-01-01','2026-01-31','salary',1,0), ('variable','歩合','1',1,'2020-01-01',NULL,'salary',1,1)")
        .execute(&pool).await.unwrap();
    let repo = IncomeRepository::new(pool);
    assert_eq!(repo.insert_recurring_for_month("2026-02").await.unwrap(), 0);
    assert_eq!(repo.insert_recurring_for_month("2026-03").await.unwrap(), 1);
}

#[tokio::test]
async fn repository_posts_when_active_period_only_partially_overlaps_month() {
    let pool = pool().await;
    sqlx::query("INSERT INTO recurring_incomes(id,name,amount,payment_day,start_date,end_date,category_id,is_active,is_variable) VALUES ('starts-after-payment','月途中開始','1',1,'2026-02-20',NULL,'salary',1,0), ('ends-before-payment','月途中終了','2',25,'2026-01-01','2026-02-10','salary',1,0)")
        .execute(&pool).await.unwrap();
    let repo = IncomeRepository::new(pool.clone());

    assert_eq!(repo.insert_recurring_for_month("2026-02").await.unwrap(), 2);
    let dates: Vec<(String, String)> = sqlx::query_as(
        "SELECT recurring_income_id, transaction_date FROM incomes ORDER BY recurring_income_id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    // 現在の挙動を固定する回帰テストであり、望ましい仕様を示すものではない。
    // 有効期間が月内で一部でも重なると、支払日が有効期間外でも計上される。
    assert_eq!(
        dates,
        vec![
            ("ends-before-payment".to_owned(), "2026-02-25".to_owned()),
            ("starts-after-payment".to_owned(), "2026-02-01".to_owned()),
        ]
    );
}

#[tokio::test]
async fn updating_auto_posted_income_preserves_list_idempotency() {
    let pool = pool().await;
    let month: String = sqlx::query_scalar("SELECT strftime('%Y-%m','now','localtime')")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO recurring_incomes(id,name,amount,payment_day,start_date,category_id,is_active,is_variable) VALUES ('salary-rule','給与','300000',25,?1,'salary',1,0)")
        .bind(format!("{month}-01")).execute(&pool).await.unwrap();
    let app = incomes::create(pool);

    let (status, first) = call(&app, "GET", "/api/incomes", None).await;
    assert_eq!(status, StatusCode::OK);
    let income = &first.unwrap()["items"][0];
    let id = income["id"].as_str().unwrap();
    let updated = json!({
        "category_id": "salary",
        "transaction_date": format!("{month}-25"),
        "amount": "310000",
        "recurring_income_id": "salary-rule",
        "description": "updated"
    });
    let (status, body) = call(&app, "PUT", &format!("/api/incomes/{id}"), Some(updated)).await;
    assert_eq!(status, StatusCode::OK, "{body:?}");

    let (status, second) = call(&app, "GET", "/api/incomes", None).await;
    assert_eq!(status, StatusCode::OK);
    let items = second.unwrap()["items"].as_array().unwrap().clone();
    assert_eq!(items.len(), 1, "{items:?}");
    assert_eq!(items[0]["amount"], "310000");
    assert_eq!(items[0]["recurring_income_id"], "salary-rule");
}

#[tokio::test]
async fn recurring_auto_post_does_not_notify_income_created_webhook() {
    let pool = pool().await;
    sqlx::query("CREATE TABLE webhook_urls(id TEXT PRIMARY KEY,created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,url TEXT NOT NULL,description TEXT,is_active INTEGER NOT NULL DEFAULT 1)")
        .execute(&pool).await.unwrap();
    let logs = Arc::new(Mutex::new(Vec::new()));
    let writer_logs = logs.clone();
    let subscriber = tracing_subscriber::fmt()
        .without_time()
        .with_ansi(false)
        .with_writer(move || LogWriter(writer_logs.clone()))
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    sqlx::query("INSERT INTO webhook_urls(id,url,is_active) VALUES('hook',?1,1)")
        .bind("http://127.0.0.1:1/income-created-test")
        .execute(&pool)
        .await
        .unwrap();
    let month: String = sqlx::query_scalar("SELECT strftime('%Y-%m','now','localtime')")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO recurring_incomes(id,name,amount,payment_day,start_date,category_id,is_active,is_variable) VALUES ('salary-rule','給与','300000',25,?1,'salary',1,0)")
        .bind(format!("{month}-01")).execute(&pool).await.unwrap();
    let app = incomes::create(pool);

    let (status, _) = call(&app, "GET", "/api/incomes", None).await;
    assert_eq!(status, StatusCode::OK);
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(!logged(&logs, "income-created-test"));

    let manual = json!({
        "category_id": "salary",
        "transaction_date": format!("{month}-15"),
        "amount": "1000",
        "description": "manual"
    });
    let (status, body) = call(&app, "POST", "/api/incomes", Some(manual)).await;
    assert_eq!(status, StatusCode::CREATED, "{body:?}");
    for _ in 0..240 {
        if logged(&logs, "income-created-test") {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!(
        "manual income creation did not attempt the configured webhook notification: {}",
        String::from_utf8_lossy(&logs.lock().unwrap())
    );
}

#[derive(Clone)]
struct LogWriter(Arc<Mutex<Vec<u8>>>);

impl Write for LogWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn logged(logs: &Arc<Mutex<Vec<u8>>>, needle: &str) -> bool {
    String::from_utf8_lossy(&logs.lock().unwrap()).contains(needle)
}
