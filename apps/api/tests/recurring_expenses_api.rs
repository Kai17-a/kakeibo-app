use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use kakeibo_app::{
    router::recurring_expenses,
    service::exchange_rate::{ExchangeRateError, ExchangeRateProvider, RateQuote},
};
use serde_json::{Value, json};
use sqlx::sqlite::SqlitePoolOptions;
use std::{future::Future, pin::Pin, sync::Arc};
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
async fn recurring_expense_rejects_non_usd_currency() {
    let p = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query("CREATE TABLE expense_categories(id TEXT PRIMARY KEY);CREATE TABLE payment_methods(id TEXT PRIMARY KEY);CREATE TABLE recurring_expenses(id TEXT PRIMARY KEY DEFAULT 're-1',created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,name TEXT NOT NULL,amount TEXT NOT NULL,payment_day INTEGER NOT NULL,start_date TEXT NOT NULL,end_date TEXT,category_id TEXT NOT NULL REFERENCES expense_categories(id),payment_method_id TEXT NOT NULL REFERENCES payment_methods(id),is_active INTEGER NOT NULL,is_variable INTEGER NOT NULL DEFAULT 0,description TEXT,foreign_amount TEXT,currency_code TEXT,exchange_rate TEXT);CREATE TABLE expenses(id TEXT PRIMARY KEY,updated_at TEXT NOT NULL DEFAULT current_timestamp,transaction_date TEXT NOT NULL,amount TEXT NOT NULL,category_id TEXT NOT NULL,payment_method_id TEXT NOT NULL,recurring_expense_id TEXT,description TEXT);INSERT INTO expense_categories VALUES('ec');INSERT INTO payment_methods VALUES('pm')")
        .execute(&p)
        .await
        .unwrap();
    let app = recurring_expenses::create(p);
    let v = json!({"name":"Cloud","amount":"1000","payment_day":1,"start_date":"2026-01-01","category_id":"ec","payment_method_id":"pm","is_active":true,"is_variable":false,"foreign_amount":"10","currency_code":"EUR"});
    assert_eq!(
        call(&app, "POST", "/api/recurring-expenses", Some(v))
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
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

struct FixedRateProvider;
impl ExchangeRateProvider for FixedRateProvider {
    fn fetch<'a>(
        &'a self,
        _date: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<RateQuote>, ExchangeRateError>> + Send + 'a>>
    {
        Box::pin(async {
            Ok(Some(RateQuote {
                rate: 150.5,
                effective_date: "2026-08-31".into(),
            }))
        })
    }
}

struct FailingRateProvider;
impl ExchangeRateProvider for FailingRateProvider {
    fn fetch<'a>(
        &'a self,
        _date: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<RateQuote>, ExchangeRateError>> + Send + 'a>>
    {
        Box::pin(async { Err(ExchangeRateError::Provider("provider unavailable".into())) })
    }
}

async fn preview_pool() -> sqlx::SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query(
        "CREATE TABLE expense_categories(id TEXT PRIMARY KEY); \
         CREATE TABLE payment_methods(id TEXT PRIMARY KEY); \
         CREATE TABLE recurring_expenses(id TEXT PRIMARY KEY,created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,name TEXT NOT NULL,amount TEXT NOT NULL,payment_day INTEGER NOT NULL,start_date TEXT NOT NULL,end_date TEXT,category_id TEXT NOT NULL,payment_method_id TEXT NOT NULL,is_active INTEGER NOT NULL,is_variable INTEGER NOT NULL,description TEXT,foreign_amount TEXT,currency_code TEXT,exchange_rate TEXT); \
         CREATE TABLE expenses(id TEXT PRIMARY KEY,created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,transaction_date TEXT NOT NULL,amount TEXT NOT NULL,category_id TEXT NOT NULL,payment_method_id TEXT NOT NULL,recurring_expense_id TEXT,description TEXT,foreign_amount TEXT,currency_code TEXT,exchange_rate TEXT,exchange_rate_date TEXT); \
         CREATE TABLE exchange_rates(target_date TEXT NOT NULL,base_currency TEXT NOT NULL,quote_currency TEXT NOT NULL,rate TEXT NOT NULL,effective_date TEXT NOT NULL,source TEXT NOT NULL,fetched_at TEXT NOT NULL DEFAULT current_timestamp,PRIMARY KEY(target_date,base_currency,quote_currency)); \
         INSERT INTO expense_categories VALUES('ec'); INSERT INTO payment_methods VALUES('pm')",
    )
    .execute(&pool)
    .await
    .unwrap();
    pool
}

async fn insert_usd_recurring(pool: &sqlx::SqlitePool) {
    sqlx::query("INSERT INTO recurring_expenses(id,name,amount,payment_day,start_date,category_id,payment_method_id,is_active,is_variable,foreign_amount,currency_code) VALUES('usd','Cloud','0',1,'2026-01-01','ec','pm',1,0,'10','USD')")
        .execute(pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn pending_months_excludes_existing_months_and_backfill_is_idempotent() {
    let pool = preview_pool().await;
    sqlx::query("INSERT INTO recurring_expenses(id,name,amount,payment_day,start_date,category_id,payment_method_id,is_active,is_variable) VALUES('jpy','Rent','80000',31,'2026-01-15','ec','pm',1,0)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO expenses(id,transaction_date,amount,category_id,payment_method_id,recurring_expense_id) VALUES('existing','2026-03-31','80000','ec','pm','jpy')")
        .execute(&pool)
        .await
        .unwrap();
    let app = recurring_expenses::create_with_exchange_rate_provider(
        pool.clone(),
        Arc::new(FixedRateProvider),
    );

    let (status, body) = call(
        &app,
        "GET",
        "/api/recurring-expenses/jpy/pending-months",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body:?}");
    let months = body.unwrap()["months"].as_array().unwrap().clone();
    assert!(!months.iter().any(|month| month == "2026-03"));
    assert_eq!(months.first().unwrap(), "2026-01");
    assert_eq!(months.last().unwrap(), "2026-08");

    let (status, body) = call(&app, "POST", "/api/recurring-expenses/jpy/backfill", None).await;
    assert_eq!(status, StatusCode::OK, "{body:?}");
    assert_eq!(body.unwrap()["created"].as_array().unwrap().len(), 7);
    let (status, body) = call(&app, "POST", "/api/recurring-expenses/jpy/backfill", None).await;
    assert_eq!(status, StatusCode::OK, "{body:?}");
    assert!(body.unwrap()["created"].as_array().unwrap().is_empty());
    let (count,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM expenses WHERE recurring_expense_id = 'jpy'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 8);
}

#[tokio::test]
async fn usd_backfill_uses_exchange_rate_and_variable_backfill_is_rejected() {
    let pool = preview_pool().await;
    sqlx::query("INSERT INTO recurring_expenses(id,name,amount,payment_day,start_date,category_id,payment_method_id,is_active,is_variable,foreign_amount,currency_code) VALUES('usd','Cloud','0',1,'2026-07-01','ec','pm',1,0,'10','USD'),('variable','Variable','0',1,'2026-01-01','ec','pm',1,1,NULL,NULL)")
        .execute(&pool)
        .await
        .unwrap();
    let app = recurring_expenses::create_with_exchange_rate_provider(
        pool.clone(),
        Arc::new(FixedRateProvider),
    );
    let (status, body) = call(&app, "POST", "/api/recurring-expenses/usd/backfill", None).await;
    assert_eq!(status, StatusCode::OK, "{body:?}");
    assert_eq!(body.unwrap()["created"].as_array().unwrap().len(), 2);
    let rows: Vec<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT strftime('%Y-%m', transaction_date), amount, exchange_rate FROM expenses WHERE recurring_expense_id = 'usd' ORDER BY transaction_date",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        rows,
        vec![
            ("2026-07".into(), "1505".into(), Some("150.5".into())),
            ("2026-08".into(), "1505".into(), Some("150.5".into()))
        ]
    );

    assert_eq!(
        call(
            &app,
            "POST",
            "/api/recurring-expenses/variable/backfill",
            None
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn usd_backfill_skips_months_when_exchange_rate_resolution_fails() {
    let pool = preview_pool().await;
    insert_usd_recurring(&pool).await;
    let app = recurring_expenses::create_with_exchange_rate_provider(
        pool.clone(),
        Arc::new(FailingRateProvider),
    );
    let (status, body) = call(&app, "POST", "/api/recurring-expenses/usd/backfill", None).await;
    assert_eq!(status, StatusCode::OK, "{body:?}");
    let body = body.unwrap();
    assert!(body["created"].as_array().unwrap().is_empty());
    assert_eq!(body["skipped"].as_array().unwrap().len(), 8);
    let (count,): (i64,) = sqlx::query_as("SELECT count(*) FROM expenses")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn exchange_rate_preview_returns_converted_amount() {
    let pool = preview_pool().await;
    insert_usd_recurring(&pool).await;
    let app =
        recurring_expenses::create_with_exchange_rate_provider(pool, Arc::new(FixedRateProvider));

    let (status, body) = call(
        &app,
        "GET",
        "/api/recurring-expenses/usd/exchange-rate?month=2026-09",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body:?}");
    let body = body.unwrap();
    assert_eq!(body["foreign_amount"], "10");
    assert_eq!(body["currency_code"], "USD");
    assert_eq!(body["exchange_rate"], "150.5");
    assert_eq!(body["exchange_rate_date"], "2026-08-31");
    assert_eq!(body["converted_amount"], "1505");
}

#[tokio::test]
async fn exchange_rate_preview_validates_resource_and_month() {
    let pool = preview_pool().await;
    insert_usd_recurring(&pool).await;
    sqlx::query("INSERT INTO recurring_expenses(id,name,amount,payment_day,start_date,category_id,payment_method_id,is_active,is_variable) VALUES('jpy','100','1000',1,'2026-01-01','ec','pm',1,0)")
        .execute(&pool)
        .await
        .unwrap();
    let app =
        recurring_expenses::create_with_exchange_rate_provider(pool, Arc::new(FixedRateProvider));

    assert_eq!(
        call(
            &app,
            "GET",
            "/api/recurring-expenses/missing/exchange-rate?month=2026-09",
            None
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(
            &app,
            "GET",
            "/api/recurring-expenses/jpy/exchange-rate?month=2026-09",
            None
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(
            &app,
            "GET",
            "/api/recurring-expenses/usd/exchange-rate?month=2026-9",
            None
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn exchange_rate_preview_returns_unprocessable_entity_on_provider_failure() {
    let pool = preview_pool().await;
    insert_usd_recurring(&pool).await;
    let app =
        recurring_expenses::create_with_exchange_rate_provider(pool, Arc::new(FailingRateProvider));

    assert_eq!(
        call(
            &app,
            "GET",
            "/api/recurring-expenses/usd/exchange-rate?month=2026-09",
            None
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}
