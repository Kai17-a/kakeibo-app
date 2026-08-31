use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use kakeibo_app::router::expense_categories;
use serde_json::{Value, json};
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use tower::ServiceExt;

async fn setup() -> (Router, SqlitePool) {
    let options = SqliteConnectOptions::new()
        .filename(":memory:")
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .unwrap();
    sqlx::query("CREATE TABLE expense_categories(id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,name TEXT NOT NULL,description TEXT,parent_category_id TEXT REFERENCES expense_categories(id));CREATE TABLE payment_methods(id TEXT PRIMARY KEY);CREATE TABLE recurring_expenses(id TEXT PRIMARY KEY);CREATE TABLE expenses(id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,transaction_date TEXT NOT NULL,amount TEXT NOT NULL,category_id TEXT NOT NULL REFERENCES expense_categories(id),payment_method_id TEXT NOT NULL REFERENCES payment_methods(id),recurring_expense_id TEXT REFERENCES recurring_expenses(id),description TEXT)").execute(&pool).await.unwrap();
    (expense_categories::create(pool.clone()), pool)
}

async fn app() -> Router {
    setup().await.0
}

async fn call(
    app: &Router,
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

async fn create(app: &Router, name: &str, parent: Option<&str>) -> Value {
    let (status, body) = call(
        app,
        "POST",
        "/api/expense-categories",
        Some(json!({"name": name, "description": null, "parent_category_id": parent})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body:?}");
    body.unwrap()
}

#[tokio::test]
async fn supports_and_validates_expense_category_hierarchy() {
    let app = app().await;
    let parent = create(&app, "Food", None).await;
    assert_eq!(parent["parent_category_id"], Value::Null);
    let parent_id = parent["id"].as_str().unwrap();
    let child = create(&app, "Dining", Some(parent_id)).await;
    let child_id = child["id"].as_str().unwrap();
    assert_eq!(child["parent_category_id"], parent_id);

    let (status, detail) = call(
        &app,
        "GET",
        &format!("/api/expense-categories/{child_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail.unwrap()["parent_category_id"], parent_id);
    let list = call(&app, "GET", "/api/expense-categories", None)
        .await
        .1
        .unwrap();
    assert!(
        list["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item.get("parent_category_id").is_some())
    );

    assert_eq!(
        call(
            &app,
            "POST",
            "/api/expense-categories",
            Some(json!({"name":"Invalid","description":null,"parent_category_id":"missing"}))
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(
            &app,
            "PUT",
            &format!("/api/expense-categories/{parent_id}"),
            Some(json!({"name":"Food","description":null,"parent_category_id":parent_id}))
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/expense-categories",
            Some(json!({"name":"Nested","description":null,"parent_category_id":child_id}))
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );

    let other = create(&app, "Other", None).await;
    let other_id = other["id"].as_str().unwrap();
    assert_eq!(
        call(
            &app,
            "PUT",
            &format!("/api/expense-categories/{parent_id}"),
            Some(json!({"name":"Food","description":null,"parent_category_id":other_id}))
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let (status, changed) = call(
        &app,
        "PUT",
        &format!("/api/expense-categories/{child_id}"),
        Some(json!({"name":"Dining","description":null,"parent_category_id":other_id})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(changed.unwrap()["parent_category_id"], other_id);
    let (status, cleared) = call(
        &app,
        "PUT",
        &format!("/api/expense-categories/{child_id}"),
        Some(json!({"name":"Dining","description":null,"parent_category_id":null})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cleared.unwrap()["parent_category_id"], Value::Null);

    let _child = create(&app, "Groceries", Some(parent_id)).await;
    assert_eq!(
        call(
            &app,
            "DELETE",
            &format!("/api/expense-categories/{parent_id}"),
            None
        )
        .await
        .0,
        StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[tokio::test]
async fn deletes_an_unused_expense_category() {
    let app = app().await;
    let category = create(&app, "Unused", None).await;
    let id = category["id"].as_str().unwrap();
    assert_eq!(
        call(
            &app,
            "DELETE",
            &format!("/api/expense-categories/{id}"),
            None
        )
        .await
        .0,
        StatusCode::NO_CONTENT
    );
}

#[tokio::test]
async fn rejects_deleting_an_expense_category_referenced_by_an_expense() {
    let (app, pool) = setup().await;
    let category = create(&app, "Food", None).await;
    let id = category["id"].as_str().unwrap();
    sqlx::query("INSERT INTO payment_methods(id) VALUES('cash');INSERT INTO expenses(transaction_date,amount,category_id,payment_method_id) VALUES('2026-08-31','1000',?,'cash')")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();

    assert_eq!(
        call(
            &app,
            "DELETE",
            &format!("/api/expense-categories/{id}"),
            None
        )
        .await
        .0,
        StatusCode::INTERNAL_SERVER_ERROR
    );
}
