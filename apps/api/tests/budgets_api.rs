use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use kakeibo_app::router::budgets;
use serde_json::{Value, json};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use tower::ServiceExt;

async fn setup() -> (axum::Router, SqlitePool) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query("CREATE TABLE expense_categories(id TEXT PRIMARY KEY); INSERT INTO expense_categories VALUES('ec'),('ec2'); CREATE TABLE budgets(id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,category_id TEXT NOT NULL UNIQUE REFERENCES expense_categories(id),amount TEXT NOT NULL);").execute(&pool).await.unwrap();
    (budgets::create(pool.clone()), pool)
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
#[tokio::test]
async fn budget_crud() {
    let (app, _) = setup().await;
    let (status, body) = call(
        &app,
        "POST",
        "/api/budgets",
        Some(json!({"category_id":"ec","amount":"1000"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body:?}");
    let id = body.unwrap()["id"].as_str().unwrap().to_owned();
    assert_eq!(
        call(&app, "GET", "/api/budgets", None)
            .await
            .1
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        call(
            &app,
            "PUT",
            &format!("/api/budgets/{id}"),
            Some(json!({"category_id":"ec2","amount":"2000"}))
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, "DELETE", &format!("/api/budgets/{id}"), None)
            .await
            .0,
        StatusCode::NO_CONTENT
    );
}
#[tokio::test]
async fn duplicate_category_is_bad_request() {
    let (app, _) = setup().await;
    let body = json!({"category_id":"ec","amount":"1"});
    assert_eq!(
        call(&app, "POST", "/api/budgets", Some(body.clone()))
            .await
            .0,
        StatusCode::CREATED
    );
    assert_eq!(
        call(&app, "POST", "/api/budgets", Some(body)).await.0,
        StatusCode::BAD_REQUEST
    );
}
#[tokio::test]
async fn missing_category_is_bad_request() {
    let (app, _) = setup().await;
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/budgets",
            Some(json!({"category_id":"missing","amount":"1"}))
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
}
