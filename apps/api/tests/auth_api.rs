use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
    routing::{get, post},
};
use kakeibo_app::{
    auth::{
        self, AuthState,
        config::AuthConfig,
        rate_limit::MAX_FAILURES,
        session_cookie::{COOKIE_NAME, digest},
    },
    router,
    service::auth::SESSION_REFRESH_INTERVAL_SECONDS,
};
use serde_json::{Value, json};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use tower::ServiceExt;

const ORIGIN: &str = "https://kakeibo.example.com";
const USERNAME: &str = "kaito";
const PASSWORD: &str = "correct horse battery staple";

struct TestApp {
    app: Router,
    pool: SqlitePool,
}

fn app_with(state: AuthState) -> Router {
    let routes = router::auth::create(state.clone())
        .route("/api/budgets", get(|| async { "budgets" }))
        .route("/api/budgets", post(|| async { "created" }))
        .merge(router::health::create())
        .route("/docs", get(|| async { "docs" }))
        .fallback(|| async { "spa" });
    auth::protect(routes, state)
}

async fn local_app() -> TestApp {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    auth::database::migrate(&pool).await.unwrap();
    let config = AuthConfig::from_lookup(|key| match key {
        "KAKEIBO_AUTH_MODE" => Some("local".to_owned()),
        "KAKEIBO_PUBLIC_URL" => Some(ORIGIN.to_owned()),
        _ => None,
    })
    .unwrap();
    let state = AuthState::local(&config, pool.clone());
    state
        .service
        .as_ref()
        .unwrap()
        .add_user(USERNAME, PASSWORD)
        .await
        .unwrap();
    TestApp {
        app: app_with(state),
        pool,
    }
}

struct Reply {
    status: StatusCode,
    set_cookie: Option<String>,
    body: Option<Value>,
}

async fn send(app: &Router, request: Request<Body>) -> Reply {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let set_cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .map(|value| value.to_str().unwrap().to_owned());
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    Reply {
        status,
        set_cookie,
        body: serde_json::from_slice(&bytes).ok(),
    }
}

fn get_request(uri: &str, cookie: Option<&str>) -> Request<Body> {
    let mut builder = Request::get(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    builder.body(Body::empty()).unwrap()
}

fn post_request(uri: &str, cookie: Option<&str>, body: Option<Value>) -> Request<Body> {
    let mut builder = Request::post(uri).header(header::ORIGIN, ORIGIN);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    match body {
        Some(body) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    }
}

async fn login(app: &Router, username: &str, password: &str) -> Reply {
    send(
        app,
        post_request(
            "/api/auth/login",
            None,
            Some(json!({ "username": username, "password": password })),
        ),
    )
    .await
}

/// Logs in and returns the `name=value` pair to send back in the `Cookie` header.
async fn session_cookie(app: &Router) -> String {
    let reply = login(app, USERNAME, PASSWORD).await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    reply
        .set_cookie
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned()
}

#[tokio::test]
async fn disabled_mode_leaves_every_route_open() {
    let app = app_with(AuthState::disabled());

    assert_eq!(
        send(&app, get_request("/api/budgets", None)).await.status,
        StatusCode::OK
    );
    let reply = send(&app, get_request("/api/auth/session", None)).await;
    assert_eq!(
        reply.body,
        Some(json!({ "mode": "none", "authenticated": true, "user": null, "passkey": false }))
    );
    assert_eq!(
        send(
            &app,
            post_request(
                "/api/auth/login",
                None,
                Some(json!({ "username": "a", "password": "b" }))
            )
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn api_and_docs_require_a_session() {
    let TestApp { app, .. } = local_app().await;

    for uri in ["/api/budgets", "/docs"] {
        let reply = send(&app, get_request(uri, None)).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{uri}");
    }
    let reply = send(&app, get_request("/api/budgets", None)).await;
    assert_eq!(
        reply.body,
        Some(json!({ "message": "authentication required" }))
    );
}

#[tokio::test]
async fn health_auth_and_static_files_stay_public() {
    let TestApp { app, .. } = local_app().await;

    for uri in ["/health", "/", "/login", "/_nuxt/app.js"] {
        assert_eq!(
            send(&app, get_request(uri, None)).await.status,
            StatusCode::OK,
            "{uri}"
        );
    }
    let reply = send(&app, get_request("/api/auth/session", None)).await;
    assert_eq!(
        reply.body,
        Some(json!({ "mode": "local", "authenticated": false, "user": null, "passkey": true }))
    );
}

#[tokio::test]
async fn login_sets_a_hardened_session_cookie() {
    let TestApp { app, .. } = local_app().await;

    let reply = login(&app, USERNAME, PASSWORD).await;

    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    let cookie = reply.set_cookie.unwrap();
    assert!(cookie.starts_with(&format!("{COOKIE_NAME}=")), "{cookie}");
    for attribute in [
        "Path=/",
        "HttpOnly",
        "SameSite=Lax",
        "Secure",
        "Max-Age=2592000",
    ] {
        assert!(cookie.contains(attribute), "{attribute} missing: {cookie}");
    }
}

#[tokio::test]
async fn session_cookie_grants_access() {
    let TestApp { app, .. } = local_app().await;
    let cookie = session_cookie(&app).await;

    let reply = send(&app, get_request("/api/budgets", Some(&cookie))).await;
    assert_eq!(reply.status, StatusCode::OK);
    let reply = send(&app, get_request("/api/auth/session", Some(&cookie))).await;
    assert_eq!(
        reply.body,
        Some(
            json!({ "mode": "local", "authenticated": true, "user": { "username": USERNAME }, "passkey": true })
        )
    );
}

#[tokio::test]
async fn username_is_case_insensitive() {
    let TestApp { app, .. } = local_app().await;

    assert_eq!(
        login(&app, "KAITO", PASSWORD).await.status,
        StatusCode::NO_CONTENT
    );
}

#[tokio::test]
async fn wrong_password_and_unknown_user_get_the_same_error() {
    let TestApp { app, .. } = local_app().await;

    let wrong_password = login(&app, USERNAME, "not the right password").await;
    let unknown_user = login(&app, "someone", PASSWORD).await;

    assert_eq!(wrong_password.status, StatusCode::UNAUTHORIZED);
    assert_eq!(unknown_user.status, StatusCode::UNAUTHORIZED);
    assert_eq!(wrong_password.body, unknown_user.body);
    assert!(wrong_password.set_cookie.is_none());
}

#[tokio::test]
async fn empty_credentials_are_rejected() {
    let TestApp { app, .. } = local_app().await;

    assert_eq!(login(&app, " ", "").await.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn repeated_failures_lock_the_account() {
    let TestApp { app, .. } = local_app().await;

    for _ in 0..MAX_FAILURES {
        assert_eq!(
            login(&app, USERNAME, "not the right password").await.status,
            StatusCode::UNAUTHORIZED
        );
    }

    assert_eq!(
        login(&app, USERNAME, PASSWORD).await.status,
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn logout_invalidates_the_session() {
    let TestApp { app, .. } = local_app().await;
    let cookie = session_cookie(&app).await;

    let reply = send(&app, post_request("/api/auth/logout", Some(&cookie), None)).await;

    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    assert!(reply.set_cookie.unwrap().contains("Max-Age=0"));
    assert_eq!(
        send(&app, get_request("/api/budgets", Some(&cookie)))
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn unknown_or_malformed_tokens_are_rejected() {
    let TestApp { app, .. } = local_app().await;
    let _ = session_cookie(&app).await;

    for cookie in [
        format!("{COOKIE_NAME}={}", "0".repeat(64)),
        format!("{COOKIE_NAME}=not-a-token"),
        "other=value".to_owned(),
    ] {
        assert_eq!(
            send(&app, get_request("/api/budgets", Some(&cookie)))
                .await
                .status,
            StatusCode::UNAUTHORIZED,
            "{cookie}"
        );
    }
}

#[tokio::test]
async fn expired_sessions_are_rejected_and_removed() {
    let TestApp { app, pool } = local_app().await;
    let cookie = session_cookie(&app).await;
    sqlx::query("UPDATE sessions SET expires_at = 0")
        .execute(&pool)
        .await
        .unwrap();

    assert_eq!(
        send(&app, get_request("/api/budgets", Some(&cookie)))
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    let remaining: i64 = sqlx::query_scalar("SELECT count(*) FROM sessions")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(remaining, 0);
}

#[tokio::test]
async fn active_sessions_are_extended_and_the_cookie_is_reissued() {
    let TestApp { app, pool } = local_app().await;
    let cookie = session_cookie(&app).await;

    let reply = send(&app, get_request("/api/budgets", Some(&cookie))).await;
    assert!(
        reply.set_cookie.is_none(),
        "fresh sessions are not rewritten"
    );

    let token = cookie.split_once('=').unwrap().1;
    sqlx::query("UPDATE sessions SET last_seen_at = last_seen_at - ?1, expires_at = expires_at - ?1 WHERE token_hash = ?2")
        .bind(SESSION_REFRESH_INTERVAL_SECONDS)
        .bind(digest(token))
        .execute(&pool)
        .await
        .unwrap();
    let before: i64 = sqlx::query_scalar("SELECT expires_at FROM sessions")
        .fetch_one(&pool)
        .await
        .unwrap();

    let reply = send(&app, get_request("/api/budgets", Some(&cookie))).await;

    assert_eq!(reply.status, StatusCode::OK);
    assert!(reply.set_cookie.unwrap().contains("Max-Age=2592000"));
    let after: i64 = sqlx::query_scalar("SELECT expires_at FROM sessions")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(after > before);
}

#[tokio::test]
async fn database_stores_only_token_digests() {
    let TestApp { app, pool } = local_app().await;
    let cookie = session_cookie(&app).await;
    let token = cookie.split_once('=').unwrap().1;

    let stored: String = sqlx::query_scalar("SELECT token_hash FROM sessions")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_ne!(stored, token);
    assert_eq!(stored, digest(token));
}

#[tokio::test]
async fn cross_origin_writes_are_rejected() {
    let TestApp { app, .. } = local_app().await;
    let cookie = session_cookie(&app).await;

    let request = Request::post("/api/budgets")
        .header(header::ORIGIN, "https://evil.example.com")
        .header(header::COOKIE, &cookie)
        .body(Body::empty())
        .unwrap();
    assert_eq!(send(&app, request).await.status, StatusCode::FORBIDDEN);

    let request = Request::post("/api/budgets")
        .header("sec-fetch-site", "cross-site")
        .header(header::COOKIE, &cookie)
        .body(Body::empty())
        .unwrap();
    assert_eq!(send(&app, request).await.status, StatusCode::FORBIDDEN);

    let request = Request::post("/api/auth/login")
        .header(header::ORIGIN, "https://evil.example.com")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "username": USERNAME, "password": PASSWORD }).to_string(),
        ))
        .unwrap();
    assert_eq!(send(&app, request).await.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn same_origin_and_originless_writes_are_allowed() {
    let TestApp { app, .. } = local_app().await;
    let cookie = session_cookie(&app).await;

    assert_eq!(
        send(&app, post_request("/api/budgets", Some(&cookie), None))
            .await
            .status,
        StatusCode::OK
    );
    let request = Request::post("/api/budgets")
        .header(header::COOKIE, &cookie)
        .body(Body::empty())
        .unwrap();
    assert_eq!(send(&app, request).await.status, StatusCode::OK);
}

#[tokio::test]
async fn logging_in_again_replaces_the_previous_session() {
    let TestApp { app, pool } = local_app().await;
    let first = session_cookie(&app).await;

    let reply = send(
        &app,
        post_request(
            "/api/auth/login",
            Some(&first),
            Some(json!({ "username": USERNAME, "password": PASSWORD })),
        ),
    )
    .await;

    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    let sessions: i64 = sqlx::query_scalar("SELECT count(*) FROM sessions")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(sessions, 1);
    assert_eq!(
        send(&app, get_request("/api/budgets", Some(&first)))
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
}
