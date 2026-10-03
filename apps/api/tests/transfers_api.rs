use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use kakeibo_app::router::transfers;
use serde_json::{Value, json};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

async fn setup() -> (sqlx::SqlitePool, axum::Router) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query("PRAGMA foreign_keys=ON; CREATE TABLE payment_methods(id TEXT PRIMARY KEY,name TEXT NOT NULL); INSERT INTO payment_methods VALUES('bank','Bank'),('nisa','NISA'); CREATE TABLE transfers(id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,transaction_date TEXT NOT NULL,amount TEXT NOT NULL,from_payment_method_id TEXT NOT NULL REFERENCES payment_methods(id),to_payment_method_id TEXT NOT NULL REFERENCES payment_methods(id),description TEXT,CHECK(CAST(amount AS INTEGER)>=1),CHECK(from_payment_method_id<>to_payment_method_id));").execute(&pool).await.unwrap();
    let app = transfers::create(pool.clone());
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

#[tokio::test]
async fn transfer_crud_and_validation() {
    let (pool, app) = setup().await;
    let input = json!({"transaction_date":"2026-10-03","amount":"30000","from_payment_method_id":"bank","to_payment_method_id":"nisa","description":"積立"});
    let (status, created) = call(&app, "POST", "/api/transfers", Some(input.clone())).await;
    assert_eq!(status, StatusCode::CREATED, "{created:?}");
    let id = created.unwrap()["id"].as_str().unwrap().to_owned();
    assert_eq!(
        call(&app, "GET", &format!("/api/transfers/{id}"), None)
            .await
            .0,
        StatusCode::OK
    );
    let mut updated = input;
    updated["amount"] = json!("40000");
    assert_eq!(
        call(&app, "PUT", &format!("/api/transfers/{id}"), Some(updated))
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, "DELETE", &format!("/api/transfers/{id}"), None)
            .await
            .0,
        StatusCode::NO_CONTENT
    );
    for invalid in [
        json!({"transaction_date":"2026/10/03","amount":"1","from_payment_method_id":"bank","to_payment_method_id":"nisa"}),
        json!({"transaction_date":"2026-10-03","amount":"0","from_payment_method_id":"bank","to_payment_method_id":"nisa"}),
        json!({"transaction_date":"2026-10-03","amount":"1","from_payment_method_id":"bank","to_payment_method_id":"bank"}),
        json!({"transaction_date":"2026-10-03","amount":"1","from_payment_method_id":"missing","to_payment_method_id":"nisa"}),
    ] {
        assert_eq!(
            call(&app, "POST", "/api/transfers", Some(invalid)).await.0,
            StatusCode::BAD_REQUEST
        );
    }

    for method in ["POST", "PUT"] {
        let uri = if method == "POST" {
            "/api/transfers".to_owned()
        } else {
            format!("/api/transfers/{id}")
        };
        for invalid in [
            json!({"transaction_date":" 2026-10-03 ","amount":"300","from_payment_method_id":"bank","to_payment_method_id":"nisa"}),
            json!({"transaction_date":"2026-10-03","amount":" 300 ","from_payment_method_id":"bank","to_payment_method_id":"nisa"}),
        ] {
            assert_eq!(
                call(&app, method, &uri, Some(invalid)).await.0,
                StatusCode::BAD_REQUEST
            );
        }
    }
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM transfers")
            .fetch_one(&pool)
            .await
            .unwrap(),
        0
    );
}

#[tokio::test]
async fn transfer_migration_applies_and_reverts() {
    static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    MIGRATOR.run(&pool).await.unwrap();
    let columns: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('payment_methods') WHERE name='is_investment'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(columns, 1);
    let tables: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='transfers'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(tables, 1);
    MIGRATOR.undo(&pool, 1).await.unwrap();
    let tables: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='transfers'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(tables, 0);
    let columns: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('payment_methods') WHERE name='is_investment'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(columns, 0);
}
