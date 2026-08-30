use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use kakeibo_app::router::recurring_incomes;
use serde_json::{Value, json};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

async fn app() -> axum::Router {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query("CREATE TABLE income_categories(id TEXT PRIMARY KEY); CREATE TABLE recurring_incomes(id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,name TEXT NOT NULL,amount TEXT NOT NULL,payment_day INTEGER NOT NULL,start_date TEXT NOT NULL,end_date TEXT,category_id TEXT NOT NULL REFERENCES income_categories(id),is_active INTEGER NOT NULL,is_variable INTEGER NOT NULL DEFAULT 0,description TEXT); INSERT INTO income_categories VALUES('salary')")
        .execute(&pool).await.unwrap();
    recurring_incomes::create(pool)
}

async fn call(
    app: &axum::Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Option<Value>) {
    let mut builder = Request::builder().method(method).uri(uri);
    let body = match body {
        Some(value) => {
            builder = builder.header("content-type", "application/json");
            Body::from(value.to_string())
        }
        None => Body::empty(),
    };
    let response = app
        .clone()
        .oneshot(builder.body(body).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (
        status,
        (!bytes.is_empty()).then(|| serde_json::from_slice(&bytes).unwrap()),
    )
}

fn input() -> Value {
    json!({"name":"給与","amount":"300000","payment_day":25,"start_date":"2026-01-01","end_date":null,"category_id":"salary","is_active":true,"is_variable":false,"description":"本業"})
}

#[tokio::test]
async fn creates_gets_lists_updates_and_deletes() {
    let app = app().await;
    let (status, created) = call(&app, "POST", "/api/recurring-incomes", Some(input())).await;
    assert_eq!(status, StatusCode::CREATED, "{created:?}");
    let created = created.unwrap();
    let id = created["id"].as_str().unwrap();
    assert_eq!(created["name"], "給与");

    let (status, found) = call(&app, "GET", &format!("/api/recurring-incomes/{id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(found.unwrap()["amount"], "300000");
    let (status, list) = call(&app, "GET", "/api/recurring-incomes", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.unwrap().as_array().unwrap().len(), 1);

    let mut updated = input();
    updated["amount"] = json!("320000");
    updated["is_variable"] = json!(true);
    let (status, body) = call(
        &app,
        "PUT",
        &format!("/api/recurring-incomes/{id}"),
        Some(updated),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_ref().unwrap()["amount"], "320000");
    assert_eq!(body.unwrap()["is_variable"], true);

    let (status, _) = call(
        &app,
        "DELETE",
        &format!("/api/recurring-incomes/{id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = call(&app, "GET", &format!("/api/recurring-incomes/{id}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn validates_required_fields_and_payment_day() {
    for invalid in [
        json!({"name":" ","amount":"1","payment_day":1,"start_date":"2026-01-01","end_date":null,"category_id":"salary","is_active":true,"is_variable":false,"description":null}),
        json!({"name":"給与","amount":"1","payment_day":0,"start_date":"2026-01-01","end_date":null,"category_id":"salary","is_active":true,"is_variable":false,"description":null}),
        json!({"name":"給与","amount":"1","payment_day":32,"start_date":"2026-01-01","end_date":null,"category_id":"salary","is_active":true,"is_variable":false,"description":null}),
    ] {
        let (status, body) = call(
            &app().await,
            "POST",
            "/api/recurring-incomes",
            Some(invalid),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body:?}");
    }
}

#[tokio::test]
async fn returns_not_found_for_missing_resources() {
    let app = app().await;
    let (status, _) = call(&app, "GET", "/api/recurring-incomes/missing", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = call(&app, "PUT", "/api/recurring-incomes/missing", Some(input())).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = call(&app, "DELETE", "/api/recurring-incomes/missing", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
