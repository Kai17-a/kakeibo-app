use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use kakeibo_app::{
    model::expenses::ExpenseUpsertRequest,
    repository::expenses::ExpenseRepository,
    router::{expenses, incomes},
};
use serde_json::{Value, json};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::{
    io::Write,
    sync::{Arc, Mutex},
    time::Duration,
};
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
    "CREATE TABLE expense_categories(id TEXT PRIMARY KEY,name TEXT NOT NULL);",
    "CREATE TABLE payment_methods(id TEXT PRIMARY KEY);",
    "CREATE TABLE recurring_expenses(id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))||'-'||hex(randomblob(2))||'-'||'4'||substr(hex(randomblob(2)),2)||'-'||substr('AB89',1+(abs(random())%4),1)||substr(hex(randomblob(2)),2)||'-'||hex(randomblob(6)))),created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,name TEXT NOT NULL,amount TEXT NOT NULL,payment_day INTEGER NOT NULL,start_date TEXT NOT NULL,end_date TEXT,category_id TEXT NOT NULL REFERENCES expense_categories(id),payment_method_id TEXT NOT NULL REFERENCES payment_methods(id),is_active INTEGER NOT NULL,is_variable INTEGER NOT NULL DEFAULT 0,description TEXT);",
    "CREATE TABLE expenses(id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))||'-'||hex(randomblob(2))||'-'||'4'||substr(hex(randomblob(2)),2)||'-'||substr('AB89',1+(abs(random())%4),1)||substr(hex(randomblob(2)),2)||'-'||hex(randomblob(6)))),created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,transaction_date TEXT NOT NULL,amount TEXT NOT NULL,category_id TEXT NOT NULL REFERENCES expense_categories(id),payment_method_id TEXT NOT NULL REFERENCES payment_methods(id),recurring_expense_id TEXT REFERENCES recurring_expenses(id),description TEXT);",
    "INSERT INTO expense_categories VALUES('ec','食費'),('ec2','娯楽費');",
    "INSERT INTO payment_methods VALUES('pm');",
    "CREATE TABLE webhook_urls(id TEXT PRIMARY KEY,created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,url TEXT NOT NULL,description TEXT,is_active INTEGER NOT NULL DEFAULT 1);",
    "CREATE TABLE webhook_url_events(webhook_url_id TEXT NOT NULL REFERENCES webhook_urls(id) ON DELETE CASCADE,event TEXT NOT NULL,PRIMARY KEY(webhook_url_id,event));",
    "CREATE TABLE budgets(id TEXT PRIMARY KEY,category_id TEXT NOT NULL UNIQUE,amount TEXT NOT NULL);",
);

fn expense(date: &str, amount: &str, category: &str) -> ExpenseUpsertRequest {
    ExpenseUpsertRequest {
        transaction_date: date.into(),
        amount: amount.into(),
        category_id: category.into(),
        payment_method_id: "pm".into(),
        recurring_expense_id: None,
        description: None,
    }
}

#[tokio::test]
async fn budget_crossing_on_create_fires_only_when_crossing_and_not_at_equal() {
    let pool = setup_pool().await;
    sqlx::query("INSERT INTO budgets VALUES('b','ec','1000')")
        .execute(&pool)
        .await
        .unwrap();
    let repo = ExpenseRepository::new(pool);
    assert!(
        repo.insert_with_budget_check(&expense("2026-07-01", "1000", "ec"))
            .await
            .unwrap()
            .budget_crossing
            .is_none()
    );
    assert!(
        repo.insert_with_budget_check(&expense("2026-07-02", "1", "ec"))
            .await
            .unwrap()
            .budget_crossing
            .is_some()
    );
    assert!(
        repo.insert_with_budget_check(&expense("2026-07-03", "1", "ec"))
            .await
            .unwrap()
            .budget_crossing
            .is_none()
    );
}

#[tokio::test]
async fn budget_crossing_on_update_evaluates_only_the_new_category_and_month_bucket() {
    let pool = setup_pool().await;
    sqlx::query("INSERT INTO budgets VALUES('b','ec','1000'),('b2','ec2','500'); INSERT INTO expenses(id,transaction_date,amount,category_id,payment_method_id) VALUES('existing','2026-08-01','500','ec2','pm')").execute(&pool).await.unwrap();
    let repo = ExpenseRepository::new(pool);
    let inserted = repo
        .insert_with_budget_check(&expense("2026-07-01", "1000", "ec"))
        .await
        .unwrap();
    let moved = repo
        .update_with_budget_check(&inserted.expense.id, &expense("2026-08-02", "1", "ec2"))
        .await
        .unwrap()
        .unwrap();
    assert!(moved.budget_crossing.is_some());
    assert!(
        repo.update_with_budget_check(&moved.expense.id, &expense("2026-08-02", "2", "ec2"))
            .await
            .unwrap()
            .unwrap()
            .budget_crossing
            .is_none()
    );
}
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
async fn create_rejects_amount_above_i64_max() {
    let p = setup_pool().await;
    let app = expenses::create(p);
    let v = json!({"transaction_date":"2026-07-22","amount":"9223372036854775808","category_id":"ec","payment_method_id":"pm","recurring_expense_id":null,"description":null});

    let (s, _) = call(&app, "POST", "/api/expenses", Some(v)).await;

    assert_eq!(s, StatusCode::BAD_REQUEST);
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
    sqlx::query(
        "INSERT INTO webhook_url_events(webhook_url_id,event) VALUES('wu-1','expense.created')",
    )
    .execute(&p)
    .await
    .unwrap();
    let app = expenses::create(p);
    let v = json!({"transaction_date":"2026-07-22","amount":"1200","category_id":"ec","payment_method_id":"pm","recurring_expense_id":null,"description":null});
    let (s, b) = call(&app, "POST", "/api/expenses", Some(v)).await;
    assert_eq!(s, StatusCode::CREATED, "{b:?}");
}

#[tokio::test]
async fn creation_notifies_only_webhooks_subscribed_to_each_event() {
    let p = setup_pool().await;
    sqlx::query(
        "CREATE TABLE income_categories(id TEXT PRIMARY KEY); \
         CREATE TABLE recurring_incomes(id TEXT PRIMARY KEY,created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,name TEXT NOT NULL,amount TEXT NOT NULL,payment_day INTEGER NOT NULL,start_date TEXT NOT NULL,end_date TEXT,category_id TEXT NOT NULL,is_active INTEGER NOT NULL,is_variable INTEGER NOT NULL DEFAULT 0,description TEXT); \
         CREATE TABLE incomes(id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,category_id TEXT NOT NULL REFERENCES income_categories(id),transaction_date TEXT NOT NULL,amount TEXT NOT NULL,payment_method_id TEXT REFERENCES payment_methods(id),recurring_income_id TEXT REFERENCES recurring_incomes(id),description TEXT); \
         INSERT INTO income_categories VALUES('salary')",
    )
    .execute(&p)
    .await
    .unwrap();
    let logs = Arc::new(Mutex::new(Vec::new()));
    let writer_logs = logs.clone();
    let subscriber = tracing_subscriber::fmt()
        .without_time()
        .with_ansi(false)
        .with_writer(move || LogWriter(writer_logs.clone()))
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    sqlx::query("INSERT INTO webhook_urls(id,url,is_active) VALUES('expense-hook',?1,1),('income-hook',?2,1)")
        .bind("http://127.0.0.1:1/expense-hook")
        .bind("http://127.0.0.1:1/income-hook")
        .execute(&p)
        .await
        .unwrap();
    sqlx::query("INSERT INTO webhook_url_events(webhook_url_id,event) VALUES('expense-hook','expense.created'),('income-hook','income.created')")
        .execute(&p)
        .await
        .unwrap();

    let expense_app = expenses::create(p.clone());
    let expense = json!({"transaction_date":"2026-07-22","amount":"1200","category_id":"ec","payment_method_id":"pm","recurring_expense_id":null,"description":null});
    let (status, body) = call(&expense_app, "POST", "/api/expenses", Some(expense)).await;
    assert_eq!(status, StatusCode::CREATED, "{body:?}");
    wait_until_logged(&logs, "expense-hook").await;
    assert!(!logged(&logs, "income-hook"));
    logs.lock().unwrap().clear();

    let income_app = incomes::create(p);
    let income = json!({"transaction_date":"2026-07-22","amount":"5000","category_id":"salary","payment_method_id":"pm","recurring_income_id":null,"description":null});
    let (status, body) = call(&income_app, "POST", "/api/incomes", Some(income)).await;
    assert_eq!(status, StatusCode::CREATED, "{body:?}");
    wait_until_logged(&logs, "income-hook").await;
    assert!(!logged(&logs, "expense-hook"));
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

async fn wait_until_logged(logs: &Arc<Mutex<Vec<u8>>>, needle: &str) {
    for _ in 0..80 {
        if logged(logs, needle) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!(
        "webhook notification was not attempted for {needle}: {}",
        String::from_utf8_lossy(&logs.lock().unwrap())
    );
}
