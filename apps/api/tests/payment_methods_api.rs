use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use kakeibo_app::router::payment_methods;
use serde_json::{Value, json};
use sqlx::sqlite::SqlitePoolOptions;
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
async fn payment_method_crud() {
    let p = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query("PRAGMA foreign_keys = ON; CREATE TABLE payment_methods(id TEXT PRIMARY KEY DEFAULT 'pm-1',created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,name TEXT NOT NULL,description TEXT,initial_balance TEXT);CREATE TABLE expenses(id TEXT PRIMARY KEY,amount TEXT NOT NULL,payment_method_id TEXT NOT NULL REFERENCES payment_methods(id));CREATE TABLE incomes(id TEXT PRIMARY KEY,amount TEXT NOT NULL,payment_method_id TEXT REFERENCES payment_methods(id))").execute(&p).await.unwrap();
    let app = payment_methods::create(p);
    let (s, _) = call(
        &app,
        "POST",
        "/api/payment-methods",
        Some(json!({"name":"Cash","description":null})),
    )
    .await;
    assert_eq!(s, StatusCode::CREATED);
    let (s, b) = call(&app, "GET", "/api/payment-methods", None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(b.unwrap()["items"][0]["name"], "Cash");
    let (s, b) = call(&app, "GET", "/api/payment-methods/pm-1", None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(b.unwrap()["balance"], Value::Null);
    let (s, _) = call(&app, "DELETE", "/api/payment-methods/pm-1", None).await;
    assert_eq!(s, StatusCode::NO_CONTENT)
}

#[tokio::test]
async fn computes_tracked_balance_from_linked_transactions() {
    let p = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query("PRAGMA foreign_keys = ON; CREATE TABLE payment_methods(id TEXT PRIMARY KEY DEFAULT 'pm-1',created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,name TEXT NOT NULL,description TEXT,initial_balance TEXT);CREATE TABLE expenses(id TEXT PRIMARY KEY,amount TEXT NOT NULL,payment_method_id TEXT NOT NULL REFERENCES payment_methods(id));CREATE TABLE incomes(id TEXT PRIMARY KEY,amount TEXT NOT NULL,payment_method_id TEXT REFERENCES payment_methods(id))").execute(&p).await.unwrap();
    let app = payment_methods::create(p.clone());
    let (status, created) = call(
        &app,
        "POST",
        "/api/payment-methods",
        Some(json!({"name":"Bank","description":null,"initial_balance":"1000"})),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created:?}");
    assert_eq!(created.unwrap()["balance"], "1000");

    sqlx::query("INSERT INTO incomes VALUES('i-1','500','pm-1');INSERT INTO expenses VALUES('e-1','1200','pm-1')")
        .execute(&p)
        .await
        .unwrap();
    let (status, body) = call(&app, "GET", "/api/payment-methods", None).await;
    assert_eq!(status, StatusCode::OK, "{body:?}");
    assert_eq!(body.unwrap()["items"][0]["balance"], "300");

    sqlx::query("INSERT INTO expenses VALUES('e-2','400','pm-1')")
        .execute(&p)
        .await
        .unwrap();
    let (status, body) = call(&app, "GET", "/api/payment-methods/pm-1", None).await;
    assert_eq!(status, StatusCode::OK, "{body:?}");
    assert_eq!(body.unwrap()["balance"], "-100");
}

#[tokio::test]
async fn rejects_invalid_initial_balances() {
    let p = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query("CREATE TABLE payment_methods(id TEXT PRIMARY KEY DEFAULT 'pm-1',created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,name TEXT NOT NULL,description TEXT,initial_balance TEXT);CREATE TABLE expenses(id TEXT PRIMARY KEY,amount TEXT NOT NULL,payment_method_id TEXT NOT NULL);CREATE TABLE incomes(id TEXT PRIMARY KEY,amount TEXT NOT NULL,payment_method_id TEXT)").execute(&p).await.unwrap();
    let app = payment_methods::create(p);
    for initial_balance in ["-1", "abc"] {
        let (status, _) = call(
            &app,
            "POST",
            "/api/payment-methods",
            Some(json!({"name":"Bank","initial_balance":initial_balance})),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{initial_balance}");
    }
}

#[tokio::test]
async fn blocks_delete_when_an_income_references_payment_method() {
    let p = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query("PRAGMA foreign_keys = ON; CREATE TABLE payment_methods(id TEXT PRIMARY KEY DEFAULT 'pm-1',created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,name TEXT NOT NULL,description TEXT,initial_balance TEXT);CREATE TABLE expenses(id TEXT PRIMARY KEY,amount TEXT NOT NULL,payment_method_id TEXT NOT NULL REFERENCES payment_methods(id));CREATE TABLE incomes(id TEXT PRIMARY KEY,amount TEXT NOT NULL,payment_method_id TEXT REFERENCES payment_methods(id));INSERT INTO payment_methods(name) VALUES('Bank');INSERT INTO incomes VALUES('i-1','500','pm-1')").execute(&p).await.unwrap();
    let app = payment_methods::create(p);
    let (status, body) = call(&app, "DELETE", "/api/payment-methods/pm-1", None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body:?}");
    assert!(
        body.unwrap()["message"]
            .as_str()
            .unwrap()
            .contains("expense or income")
    );
}
