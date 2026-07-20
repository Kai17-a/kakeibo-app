use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use kakeibo_app::router::incomes;
use serde_json::{Value, json};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use tower::ServiceExt;

async fn pool() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query(
        "CREATE TABLE income_categories (id TEXT PRIMARY KEY); \
         CREATE TABLE incomes (id TEXT PRIMARY KEY DEFAULT 'income-1', \
         created_at TEXT NOT NULL DEFAULT current_timestamp, \
         updated_at TEXT NOT NULL DEFAULT current_timestamp, category_id TEXT NOT NULL \
         REFERENCES income_categories(id), transaction_date TEXT NOT NULL, amount TEXT NOT NULL, \
         description TEXT)",
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
    let (status, body) = call(&app, "PUT", "/api/incomes/income-1", Some(updated)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.unwrap()["amount"], "310000");

    let (status, _) = call(&app, "DELETE", "/api/incomes/income-1", None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = call(&app, "GET", "/api/incomes/income-1", None).await;
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
