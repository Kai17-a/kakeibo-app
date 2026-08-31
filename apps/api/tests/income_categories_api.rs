use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use kakeibo_app::router::income_categories;
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
    sqlx::query("CREATE TABLE income_categories(id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,name TEXT NOT NULL,description TEXT,parent_category_id TEXT REFERENCES income_categories(id));CREATE TABLE recurring_incomes(id TEXT PRIMARY KEY);CREATE TABLE incomes(id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(8)))),created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,category_id TEXT NOT NULL REFERENCES income_categories(id),transaction_date TEXT NOT NULL,amount TEXT NOT NULL,recurring_income_id TEXT REFERENCES recurring_incomes(id),description TEXT)").execute(&pool).await.unwrap();
    (income_categories::create(pool.clone()), pool)
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
        "/api/income-categories",
        Some(json!({"name":name,"description":null,"parent_category_id":parent})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body:?}");
    body.unwrap()
}

#[tokio::test]
async fn supports_and_validates_income_category_hierarchy() {
    let app = app().await;
    let parent = create(&app, "Salary", None).await;
    let parent_id = parent["id"].as_str().unwrap();
    assert_eq!(parent["parent_category_id"], Value::Null);
    let child = create(&app, "Bonus", Some(parent_id)).await;
    let child_id = child["id"].as_str().unwrap();
    assert_eq!(child["parent_category_id"], parent_id);

    let (status, detail) = call(
        &app,
        "GET",
        &format!("/api/income-categories/{child_id}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail.unwrap()["parent_category_id"], parent_id);
    let list = call(&app, "GET", "/api/income-categories", None)
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
            "/api/income-categories",
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
            &format!("/api/income-categories/{parent_id}"),
            Some(json!({"name":"Salary","description":null,"parent_category_id":parent_id}))
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/income-categories",
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
            &format!("/api/income-categories/{parent_id}"),
            Some(json!({"name":"Salary","description":null,"parent_category_id":other_id}))
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    let (status, changed) = call(
        &app,
        "PUT",
        &format!("/api/income-categories/{child_id}"),
        Some(json!({"name":"Bonus","description":null,"parent_category_id":other_id})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(changed.unwrap()["parent_category_id"], other_id);
    let (status, cleared) = call(
        &app,
        "PUT",
        &format!("/api/income-categories/{child_id}"),
        Some(json!({"name":"Bonus","description":null,"parent_category_id":null})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cleared.unwrap()["parent_category_id"], Value::Null);

    let _child = create(&app, "Allowance", Some(parent_id)).await;
    assert_eq!(
        call(
            &app,
            "DELETE",
            &format!("/api/income-categories/{parent_id}"),
            None
        )
        .await
        .0,
        StatusCode::INTERNAL_SERVER_ERROR
    );
}

#[tokio::test]
async fn rejects_invalid_query() {
    let app = app().await;

    for name in ["", " "] {
        let (status, body) = call(
            &app,
            "POST",
            "/api/income-categories",
            Some(json!({ "name": name, "description": null, "parent_category_id": null })),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{name:?}: {body:?}");
    }

    for query in ["unknown=value", "sort_by=unknown", "page=0", "per_page=101"] {
        let uri = format!("/api/income-categories?{query}");
        let (status, body) = call(&app, "GET", &uri, None).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{query}: {body:?}");
    }
}

#[tokio::test]
async fn filters_sorts_and_paginates_income_categories() {
    let app = app().await;
    let alpha = create(&app, "Alpha", None).await;
    let _beta = create(&app, "Beta", None).await;
    let gamma = create(&app, "Gamma", None).await;

    let page = call(
        &app,
        "GET",
        "/api/income-categories?sort_by=name&sort_order=desc&page=2&per_page=1",
        None,
    )
    .await
    .1
    .unwrap();
    assert_eq!(page["items"][0]["name"], "Beta");
    assert_eq!(page["pagination"], json!({"page": 2, "per_page": 1}));

    let gamma_id = gamma["id"].as_str().unwrap();
    let filtered = call(
        &app,
        "GET",
        &format!("/api/income-categories?id={gamma_id}"),
        None,
    )
    .await
    .1
    .unwrap();
    assert_eq!(filtered["items"][0]["name"], "Gamma");
    assert_ne!(alpha["id"], gamma["id"]);
}

#[tokio::test]
async fn rejects_deleting_an_income_category_referenced_by_an_income() {
    let (app, pool) = setup().await;
    let category = create(&app, "Salary", None).await;
    let id = category["id"].as_str().unwrap();
    sqlx::query(
        "INSERT INTO incomes(category_id,transaction_date,amount) VALUES(?,'2026-08-31','300000')",
    )
    .bind(id)
    .execute(&pool)
    .await
    .unwrap();

    assert_eq!(
        call(
            &app,
            "DELETE",
            &format!("/api/income-categories/{id}"),
            None
        )
        .await
        .0,
        StatusCode::INTERNAL_SERVER_ERROR
    );
}
