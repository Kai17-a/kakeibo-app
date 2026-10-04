use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use chrono::{Datelike, Months, NaiveDate};
use kakeibo_app::router::{recurring_transfers, transfers};
use serde_json::{Value, json};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

async fn setup() -> (sqlx::SqlitePool, axum::Router) {
    static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    MIGRATOR.run(&pool).await.unwrap();
    sqlx::query("INSERT INTO payment_methods(id,name,is_investment) VALUES('bank','銀行',0),('nisa','NISA',1)").execute(&pool).await.unwrap();
    let app = recurring_transfers::create(pool.clone()).merge(transfers::create(pool.clone()));
    (pool, app)
}

async fn call(
    app: &axum::Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Option<Value>) {
    let mut request = Request::builder().method(method).uri(uri);
    let body = if let Some(value) = body {
        request = request.header("content-type", "application/json");
        Body::from(value.to_string())
    } else {
        Body::empty()
    };
    let response = app
        .clone()
        .oneshot(request.body(body).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (
        status,
        (!bytes.is_empty()).then(|| serde_json::from_slice(&bytes).unwrap()),
    )
}

fn input(start_date: &str) -> Value {
    json!({"name":"つみたてNISA","amount":"30000","payment_day":31,"start_date":start_date,"end_date":null,"from_payment_method_id":"bank","to_payment_method_id":"nisa","is_active":true,"description":"積立"})
}

async fn current_month(pool: &sqlx::SqlitePool) -> NaiveDate {
    let value: String = sqlx::query_scalar("SELECT date('now', 'localtime', 'start of month')")
        .fetch_one(pool)
        .await
        .unwrap();
    NaiveDate::parse_from_str(&value, "%Y-%m-%d").unwrap()
}

async fn get_transfers(app: &axum::Router) -> Value {
    let (status, body) = call(app, "GET", "/api/transfers", None).await;
    assert_eq!(status, StatusCode::OK, "{body:?}");
    body.unwrap()
}

async fn insert_recurring(
    pool: &sqlx::SqlitePool,
    id: &str,
    start: &str,
    end: Option<&str>,
    active: bool,
    payment_day: i64,
) {
    sqlx::query("INSERT INTO recurring_transfers(id,name,amount,payment_day,start_date,end_date,from_payment_method_id,to_payment_method_id,is_active) VALUES(?,?,'1000',?,?,?,'bank','nisa',?)")
        .bind(id).bind(id).bind(payment_day).bind(start).bind(end).bind(active)
        .execute(pool).await.unwrap();
}

#[tokio::test]
async fn crud_validation_and_posted_transfer_deletion_conflict_are_supported() {
    let (pool, app) = setup().await;
    let start = current_month(&pool).await.to_string();
    let (status, created) = call(
        &app,
        "POST",
        "/api/recurring-transfers",
        Some(input(&start)),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created:?}");
    let id = created.unwrap()["id"].as_str().unwrap().to_owned();
    get_transfers(&app).await;
    let (status, body) = call(
        &app,
        "DELETE",
        &format!("/api/recurring-transfers/{id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        body.unwrap()["message"],
        "計上済みの振替明細があるため削除できません。停止する場合は無効にするか終了日を設定してください。"
    );

    let invalid = json!({"name":"積立","amount":"0","payment_day":32,"start_date":"bad","end_date":null,"from_payment_method_id":"bank","to_payment_method_id":"bank","is_active":true,"description":null});
    let (status, _) = call(&app, "POST", "/api/recurring-transfers", Some(invalid)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn inactive_recurring_transfer_is_not_posted() {
    let (pool, app) = setup().await;
    insert_recurring(&pool, "inactive", "2020-01-01", None, false, 1).await;
    assert_eq!(get_transfers(&app).await.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn months_before_start_and_after_end_are_not_posted() {
    let (pool, app) = setup().await;
    insert_recurring(&pool, "bounded", "2024-03-01", Some("2024-05-31"), true, 1).await;
    get_transfers(&app).await;
    let months: Vec<String> = sqlx::query_scalar(
        "SELECT substr(transaction_date, 1, 7) FROM transfers ORDER BY transaction_date",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(months, ["2024-03", "2024-04", "2024-05"]);
}

#[tokio::test]
async fn partial_start_and_end_months_are_included() {
    let (pool, app) = setup().await;
    insert_recurring(&pool, "partial", "2024-03-31", Some("2024-05-01"), true, 15).await;
    get_transfers(&app).await;
    let dates: Vec<String> =
        sqlx::query_scalar("SELECT transaction_date FROM transfers ORDER BY transaction_date")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(dates, ["2024-03-15", "2024-04-15", "2024-05-15"]);
}

#[tokio::test]
async fn future_months_are_not_posted() {
    let (pool, app) = setup().await;
    let future = current_month(&pool)
        .await
        .checked_add_months(Months::new(1))
        .unwrap();
    insert_recurring(&pool, "future", &future.to_string(), None, true, 1).await;
    assert_eq!(get_transfers(&app).await.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn payment_day_is_clamped_for_common_leap_and_thirty_one_day_months() {
    let (pool, app) = setup().await;
    for (id, month) in [
        ("common", "2023-02-01"),
        ("leap", "2024-02-01"),
        ("long", "2024-01-01"),
    ] {
        insert_recurring(&pool, id, month, Some(month), true, 31).await;
    }
    get_transfers(&app).await;
    let dates: Vec<String> =
        sqlx::query_scalar("SELECT transaction_date FROM transfers ORDER BY transaction_date")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(dates, ["2023-02-28", "2024-01-31", "2024-02-29"]);
}

#[tokio::test]
async fn deleted_posted_transfer_is_posted_again() {
    let (pool, app) = setup().await;
    let start = current_month(&pool).await.to_string();
    insert_recurring(&pool, "repost", &start, Some(&start), true, 1).await;
    get_transfers(&app).await;
    sqlx::query("DELETE FROM transfers WHERE recurring_transfer_id = 'repost'")
        .execute(&pool)
        .await
        .unwrap();
    get_transfers(&app).await;
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM transfers WHERE recurring_transfer_id = 'repost'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn all_months_from_start_through_current_month_are_posted_at_once() {
    let (pool, app) = setup().await;
    let current = current_month(&pool).await;
    let start = current.checked_sub_months(Months::new(59)).unwrap();
    insert_recurring(&pool, "five-years", &start.to_string(), None, true, 1).await;
    let transfers = get_transfers(&app).await;
    assert_eq!(transfers.as_array().unwrap().len(), 60);
    let latest: String = sqlx::query_scalar("SELECT max(transaction_date) FROM transfers")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        latest,
        format!("{}-{:02}-01", current.year(), current.month())
    );
}

#[tokio::test]
async fn posting_failure_rolls_back_every_month() {
    let (pool, app) = setup().await;
    insert_recurring(&pool, "rollback", "2024-01-01", Some("2024-03-01"), true, 1).await;
    sqlx::query("CREATE TRIGGER fail_second_month BEFORE INSERT ON transfers WHEN NEW.recurring_transfer_id = 'rollback' AND NEW.transaction_date = '2024-02-01' BEGIN SELECT RAISE(ABORT, 'test failure'); END").execute(&pool).await.unwrap();
    let (status, _) = call(&app, "GET", "/api/transfers", None).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM transfers")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}
