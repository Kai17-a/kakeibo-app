use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use kakeibo_app::router::webhook_urls;
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
        (!bytes.is_empty())
            .then(|| serde_json::from_slice(&bytes).ok())
            .flatten(),
    )
}
async fn setup_pool() -> sqlx::SqlitePool {
    let p = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query("CREATE TABLE webhook_urls(id TEXT PRIMARY KEY DEFAULT 'wu-1',created_at TEXT NOT NULL DEFAULT current_timestamp,updated_at TEXT NOT NULL DEFAULT current_timestamp,url TEXT NOT NULL,description TEXT,is_active INTEGER NOT NULL DEFAULT 1); CREATE TABLE webhook_url_events(webhook_url_id TEXT NOT NULL REFERENCES webhook_urls(id) ON DELETE CASCADE,event TEXT NOT NULL,PRIMARY KEY(webhook_url_id,event))").execute(&p).await.unwrap();
    p
}
#[tokio::test]
async fn webhook_url_crud() {
    let p = setup_pool().await;
    let app = webhook_urls::create(p);
    let v = json!({"url":"https://example.com/hook","description":"通知先1","is_active":true,"events":["expense.created"]});
    let (s, b) = call(&app, "POST", "/api/webhook-urls", Some(v)).await;
    assert_eq!(s, StatusCode::CREATED, "{b:?}");
    assert_eq!(b.as_ref().unwrap()["is_active"], true);
    assert_eq!(b.as_ref().unwrap()["events"], json!(["expense.created"]));

    let (s, b) = call(&app, "GET", "/api/webhook-urls", None).await;
    assert_eq!(s, StatusCode::OK);
    let b = b.unwrap();
    assert_eq!(b[0]["url"], "https://example.com/hook");
    assert_eq!(b[0]["is_active"], true);

    let v = json!({"url":"https://example.com/hook2","description":null,"is_active":false,"events":["expense.created","income.created","budget.exceeded"]});
    let (s, b) = call(&app, "PUT", "/api/webhook-urls/wu-1", Some(v)).await;
    assert_eq!(s, StatusCode::OK, "{b:?}");
    assert_eq!(b.as_ref().unwrap()["is_active"], false);
    assert_eq!(
        b.as_ref().unwrap()["events"],
        json!(["expense.created", "income.created", "budget.exceeded"])
    );

    let (s, _) = call(&app, "DELETE", "/api/webhook-urls/wu-1", None).await;
    assert_eq!(s, StatusCode::NO_CONTENT);
    let (s, _) = call(&app, "GET", "/api/webhook-urls/wu-1", None).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
}
#[tokio::test]
async fn rejects_url_without_http_scheme() {
    let p = setup_pool().await;
    let app = webhook_urls::create(p);
    let v = json!({"url":"example.com","description":null,"is_active":true,"events":["expense.created"]});
    let (s, _) = call(&app, "POST", "/api/webhook-urls", Some(v)).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn accepts_known_events_and_removes_duplicates() {
    let p = setup_pool().await;
    let app = webhook_urls::create(p);
    let v = json!({
        "url":"https://example.com/hook",
        "description":null,
        "is_active":true,
        "events":["income.created","income.created","budget.exceeded"]
    });

    let (s, b) = call(&app, "POST", "/api/webhook-urls", Some(v)).await;

    assert_eq!(s, StatusCode::CREATED, "{b:?}");
    assert_eq!(
        b.unwrap()["events"],
        json!(["income.created", "budget.exceeded"])
    );
}

#[tokio::test]
async fn get_reads_back_persisted_events_after_create_and_update() {
    let p = setup_pool().await;
    let app = webhook_urls::create(p);
    let created_events = json!(["expense.created", "budget.exceeded"]);
    let v = json!({
        "url":"https://example.com/hook",
        "description":null,
        "is_active":true,
        "events":created_events
    });
    let (s, _) = call(&app, "POST", "/api/webhook-urls", Some(v)).await;
    assert_eq!(s, StatusCode::CREATED);

    let (s, b) = call(&app, "GET", "/api/webhook-urls/wu-1", None).await;
    assert_eq!(s, StatusCode::OK, "{b:?}");
    let mut persisted = b.unwrap()["events"].as_array().unwrap().clone();
    let mut expected = created_events.as_array().unwrap().clone();
    persisted.sort_by_key(Value::to_string);
    expected.sort_by_key(Value::to_string);
    assert_eq!(persisted, expected);

    let updated_events = json!(["income.created", "expense.created"]);
    let v = json!({
        "url":"https://example.com/updated",
        "description":null,
        "is_active":true,
        "events":updated_events
    });
    let (s, _) = call(&app, "PUT", "/api/webhook-urls/wu-1", Some(v)).await;
    assert_eq!(s, StatusCode::OK);

    let (s, b) = call(&app, "GET", "/api/webhook-urls/wu-1", None).await;
    assert_eq!(s, StatusCode::OK, "{b:?}");
    let mut persisted = b.unwrap()["events"].as_array().unwrap().clone();
    let mut expected = updated_events.as_array().unwrap().clone();
    persisted.sort_by_key(Value::to_string);
    expected.sort_by_key(Value::to_string);
    assert_eq!(persisted, expected);
}

#[tokio::test]
async fn rejects_unknown_or_empty_events() {
    for (events, expected_status) in [
        (json!([]), StatusCode::BAD_REQUEST),
        (json!(["expense.deleted"]), StatusCode::UNPROCESSABLE_ENTITY),
    ] {
        let p = setup_pool().await;
        let app = webhook_urls::create(p);
        let v = json!({
            "url":"https://example.com/hook",
            "description":null,
            "is_active":true,
            "events":events
        });

        let (s, _) = call(&app, "POST", "/api/webhook-urls", Some(v)).await;

        assert_eq!(s, expected_status);
    }
}

#[tokio::test]
async fn migration_subscribes_existing_webhook_to_all_known_events() {
    let p = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query("CREATE TABLE webhook_urls(id TEXT PRIMARY KEY,url TEXT NOT NULL); INSERT INTO webhook_urls(id,url) VALUES('legacy','https://example.com/hook')")
        .execute(&p)
        .await
        .unwrap();

    sqlx::query(include_str!(
        "../migrations/0008__webhook_url_events.up.sql"
    ))
    .execute(&p)
    .await
    .unwrap();
    let events: Vec<String> = sqlx::query_scalar(
        "SELECT event FROM webhook_url_events WHERE webhook_url_id = 'legacy' ORDER BY event",
    )
    .fetch_all(&p)
    .await
    .unwrap();

    assert_eq!(
        events,
        vec!["budget.exceeded", "expense.created", "income.created"]
    );
}
