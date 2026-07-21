use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use kakeibo_app::router::income_categories;
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
        "CREATE TABLE income_categories (\
         id TEXT PRIMARY KEY DEFAULT 'category-1', \
         created_at TEXT NOT NULL DEFAULT current_timestamp, \
         updated_at TEXT NOT NULL DEFAULT current_timestamp, \
         name TEXT NOT NULL, description TEXT)",
    )
    .execute(&pool)
    .await
    .unwrap();
    pool
}

async fn app() -> Router {
    income_categories::create(pool().await)
}

async fn app_with_categories() -> Router {
    let pool = pool().await;
    sqlx::query(
        "INSERT INTO income_categories (id, name, description) VALUES \
         ('category-a', 'Alpha', NULL), \
         ('category-b', 'Beta', 'second'), \
         ('category-c', 'Gamma', NULL)",
    )
    .execute(&pool)
    .await
    .unwrap();
    income_categories::create(pool)
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
async fn income_category_crud_lifecycle() {
    let app = app().await;
    let input = json!({ "name": "Salary", "description": "Monthly salary" });

    let (status, created) = call(&app, "POST", "/api/income-categories", Some(input.clone())).await;
    assert_eq!(status, StatusCode::CREATED, "{created:?}");
    assert_eq!(created.as_ref().unwrap()["name"], "Salary");

    let (status, list) = call(&app, "GET", "/api/income-categories", None).await;
    assert_eq!(status, StatusCode::OK);
    let list = list.unwrap();
    assert_eq!(list["items"].as_array().unwrap().len(), 1);
    assert_eq!(list["pagination"], json!({ "page": 1, "per_page": 50 }));

    let updated = json!({ "name": "Bonus", "description": null });
    let (status, body) = call(
        &app,
        "PUT",
        "/api/income-categories/category-1",
        Some(updated),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.unwrap()["name"], "Bonus");

    let (status, _) = call(&app, "DELETE", "/api/income-categories/category-1", None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (status, _) = call(&app, "GET", "/api/income-categories/category-1", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn filters_sorts_and_paginates_income_categories() {
    let app = app_with_categories().await;

    let uri = "/api/income-categories?sort_by=name&sort_order=desc&page=2&per_page=1";
    let (status, page) = call(&app, "GET", uri, None).await;
    assert_eq!(status, StatusCode::OK);
    let page = page.unwrap();
    assert_eq!(page["items"][0]["id"], "category-b");
    assert_eq!(page["pagination"], json!({ "page": 2, "per_page": 1 }));

    let (status, filtered) = call(&app, "GET", "/api/income-categories?id=category-c", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(filtered.unwrap()["items"][0]["name"], "Gamma");
}

#[tokio::test]
async fn rejects_invalid_income_category_requests() {
    let app = app_with_categories().await;

    let (status, _) = call(
        &app,
        "POST",
        "/api/income-categories",
        Some(json!({ "name": " ", "description": null })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    for query in ["unknown=value", "sort_by=unknown", "page=0", "per_page=101"] {
        let uri = format!("/api/income-categories?{query}");
        let (status, body) = call(&app, "GET", &uri, None).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{query}: {body:?}");
    }
}
