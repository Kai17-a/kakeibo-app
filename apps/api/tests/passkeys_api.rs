//! Passkey registration and login, driven by webauthn-authenticator-rs's software authenticator.
//!
//! The software authenticator cannot store discoverable credentials and never returns a user
//! handle, so the tests adapt the ceremony on the client side the way a real platform
//! authenticator behaves: registration drops the resident-key requirement, and login lists the
//! credential and attaches the user handle (which is not covered by the signature).

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
    routing::get,
};
use kakeibo_app::{
    auth::{self, AuthState, config::AuthConfig},
    router,
};
use serde_json::{Value, json};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use tower::ServiceExt;
use webauthn_authenticator_rs::{WebauthnAuthenticator, softpasskey::SoftPasskey};
use webauthn_rs::prelude::{
    CreationChallengeResponse, PublicKeyCredential, RegisterPublicKeyCredential,
    RequestChallengeResponse, Url, Uuid,
};
use webauthn_rs_proto::{AllowCredentials, ResidentKeyRequirement};

const ORIGIN: &str = "https://kakeibo.example.com";
const PASSWORD: &str = "correct horse battery staple";

struct TestApp {
    app: Router,
    pool: SqlitePool,
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
    for username in ["kaito", "hanako"] {
        state
            .service
            .as_ref()
            .unwrap()
            .add_user(username, PASSWORD)
            .await
            .unwrap();
    }
    let routes = router::auth::create(state.clone()).route("/api/budgets", get(|| async { "ok" }));
    TestApp {
        app: auth::protect(routes, state),
        pool,
    }
}

async fn send(
    app: &Router,
    method: &str,
    uri: &str,
    cookie: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Option<String>, Option<Value>) {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::ORIGIN, ORIGIN);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let request = match body {
        Some(body) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string())),
        None => builder.body(Body::empty()),
    }
    .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let cookie = response.headers().get(header::SET_COOKIE).map(|value| {
        value
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_owned()
    });
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, cookie, serde_json::from_slice(&bytes).ok())
}

async fn password_login(app: &Router, username: &str) -> String {
    let (status, cookie, _) = send(
        app,
        "POST",
        "/api/auth/login",
        None,
        Some(json!({ "username": username, "password": PASSWORD })),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    cookie.unwrap()
}

async fn user_id(pool: &SqlitePool, username: &str) -> String {
    sqlx::query_scalar("SELECT id FROM users WHERE username = ?1")
        .bind(username)
        .fetch_one(pool)
        .await
        .unwrap()
}

fn origin() -> Url {
    Url::parse(ORIGIN).unwrap()
}

/// Registers a passkey for the signed-in user and returns the credential the server stored.
async fn register(
    app: &Router,
    cookie: &str,
    authenticator: &mut SoftPasskey,
) -> RegisterPublicKeyCredential {
    let (status, _, body) = send(
        app,
        "POST",
        "/api/passkeys/registration",
        Some(cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let body = body.unwrap();
    let selection = &body["options"]["publicKey"]["authenticatorSelection"];
    assert_eq!(selection["residentKey"], "required");
    assert_eq!(selection["userVerification"], "required");

    let mut options: CreationChallengeResponse =
        serde_json::from_value(body["options"].clone()).unwrap();
    let selection = options.public_key.authenticator_selection.as_mut().unwrap();
    selection.resident_key = Some(ResidentKeyRequirement::Discouraged);
    selection.require_resident_key = false;
    let credential = authenticator.do_registration(origin(), options).unwrap();

    let (status, _, body) = send(
        app,
        "POST",
        "/api/passkeys",
        Some(cookie),
        Some(json!({
            "challenge_id": body["challenge_id"],
            "name": " iPhone ",
            "credential": credential,
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body:?}");
    let body = body.unwrap();
    assert_eq!(body["name"], "iPhone");
    assert_eq!(body["last_used_at"], Value::Null);
    credential
}

/// Answers a login challenge as a discoverable authenticator holding `credential`.
async fn assertion(
    app: &Router,
    authenticator: &mut SoftPasskey,
    credential: &RegisterPublicKeyCredential,
    user_handle: Uuid,
) -> (String, PublicKeyCredential) {
    let (status, _, body) = send(app, "POST", "/api/auth/passkey/challenge", None, None).await;
    assert_eq!(status, StatusCode::OK);
    let body = body.unwrap();
    assert!(
        body["options"]["publicKey"]["allowCredentials"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(body["options"].get("mediation").is_none());

    let mut options: RequestChallengeResponse =
        serde_json::from_value(body["options"].clone()).unwrap();
    options.public_key.allow_credentials = vec![AllowCredentials {
        type_: "public-key".to_owned(),
        id: credential.raw_id.clone(),
        transports: None,
    }];
    let mut response = authenticator.do_authentication(origin(), options).unwrap();
    response.response.user_handle = Some(user_handle.as_bytes().to_vec());
    (body["challenge_id"].as_str().unwrap().to_owned(), response)
}

async fn passkey_login(
    app: &Router,
    challenge_id: &str,
    credential: &PublicKeyCredential,
) -> (StatusCode, Option<String>) {
    let (status, cookie, _) = send(
        app,
        "POST",
        "/api/auth/passkey/login",
        None,
        Some(json!({ "challenge_id": challenge_id, "credential": credential })),
    )
    .await;
    (status, cookie)
}

#[tokio::test]
async fn session_reports_passkey_support() {
    let TestApp { app, .. } = local_app().await;

    let (_, _, body) = send(&app, "GET", "/api/auth/session", None, None).await;

    assert_eq!(body.unwrap()["passkey"], true);
}

#[tokio::test]
async fn registers_a_passkey_and_logs_in_with_it() {
    let TestApp { app, pool } = local_app().await;
    let cookie = password_login(&app, "kaito").await;
    let mut authenticator = SoftPasskey::new(true);
    let credential = register(&app, &cookie, &mut authenticator).await;
    let user = Uuid::parse_str(&user_id(&pool, "kaito").await).unwrap();

    let (challenge_id, response) = assertion(&app, &mut authenticator, &credential, user).await;
    let (status, session) = passkey_login(&app, &challenge_id, &response).await;

    assert_eq!(status, StatusCode::NO_CONTENT);
    let session = session.unwrap();
    let (status, _, _) = send(&app, "GET", "/api/budgets", Some(&session), None).await;
    assert_eq!(status, StatusCode::OK);
    let (_, _, body) = send(&app, "GET", "/api/auth/session", Some(&session), None).await;
    assert_eq!(body.unwrap()["user"]["username"], "kaito");

    let (_, _, body) = send(&app, "GET", "/api/passkeys", Some(&session), None).await;
    let passkeys = body.unwrap();
    assert_eq!(passkeys.as_array().unwrap().len(), 1);
    assert!(passkeys[0]["last_used_at"].is_i64());
}

#[tokio::test]
async fn a_login_challenge_can_be_answered_only_once() {
    let TestApp { app, pool } = local_app().await;
    let cookie = password_login(&app, "kaito").await;
    let mut authenticator = SoftPasskey::new(true);
    let credential = register(&app, &cookie, &mut authenticator).await;
    let user = Uuid::parse_str(&user_id(&pool, "kaito").await).unwrap();
    let (challenge_id, response) = assertion(&app, &mut authenticator, &credential, user).await;

    assert_eq!(
        passkey_login(&app, &challenge_id, &response).await.0,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        passkey_login(&app, &challenge_id, &response).await.0,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn rejects_a_passkey_presented_for_another_user() {
    let TestApp { app, pool } = local_app().await;
    let cookie = password_login(&app, "kaito").await;
    let mut authenticator = SoftPasskey::new(true);
    let credential = register(&app, &cookie, &mut authenticator).await;
    let other = Uuid::parse_str(&user_id(&pool, "hanako").await).unwrap();

    let (challenge_id, response) = assertion(&app, &mut authenticator, &credential, other).await;
    let (status, cookie) = passkey_login(&app, &challenge_id, &response).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(cookie.is_none());
}

#[tokio::test]
async fn rejects_a_tampered_signature() {
    let TestApp { app, pool } = local_app().await;
    let cookie = password_login(&app, "kaito").await;
    let mut authenticator = SoftPasskey::new(true);
    let credential = register(&app, &cookie, &mut authenticator).await;
    let user = Uuid::parse_str(&user_id(&pool, "kaito").await).unwrap();
    let (challenge_id, mut response) = assertion(&app, &mut authenticator, &credential, user).await;
    let last = response.response.signature.len() - 1;
    response.response.signature[last] ^= 0xff;

    assert_eq!(
        passkey_login(&app, &challenge_id, &response).await.0,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn a_deleted_passkey_can_no_longer_log_in() {
    let TestApp { app, pool } = local_app().await;
    let cookie = password_login(&app, "kaito").await;
    let mut authenticator = SoftPasskey::new(true);
    let credential = register(&app, &cookie, &mut authenticator).await;
    let (_, _, body) = send(&app, "GET", "/api/passkeys", Some(&cookie), None).await;
    let id = body.unwrap()[0]["id"].as_str().unwrap().to_owned();

    let (status, _, _) = send(
        &app,
        "DELETE",
        &format!("/api/passkeys/{id}"),
        Some(&cookie),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let user = Uuid::parse_str(&user_id(&pool, "kaito").await).unwrap();
    let (challenge_id, response) = assertion(&app, &mut authenticator, &credential, user).await;
    assert_eq!(
        passkey_login(&app, &challenge_id, &response).await.0,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn passkeys_are_private_to_their_owner() {
    let TestApp { app, .. } = local_app().await;
    let kaito = password_login(&app, "kaito").await;
    let mut authenticator = SoftPasskey::new(true);
    register(&app, &kaito, &mut authenticator).await;
    let (_, _, body) = send(&app, "GET", "/api/passkeys", Some(&kaito), None).await;
    let id = body.unwrap()[0]["id"].as_str().unwrap().to_owned();

    let hanako = password_login(&app, "hanako").await;
    let (_, _, body) = send(&app, "GET", "/api/passkeys", Some(&hanako), None).await;
    assert_eq!(body.unwrap(), json!([]));
    let (status, _, _) = send(
        &app,
        "DELETE",
        &format!("/api/passkeys/{id}"),
        Some(&hanako),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_registration_challenge_belongs_to_the_user_who_started_it() {
    let TestApp { app, .. } = local_app().await;
    let kaito = password_login(&app, "kaito").await;
    let hanako = password_login(&app, "hanako").await;
    let (_, _, body) = send(
        &app,
        "POST",
        "/api/passkeys/registration",
        Some(&kaito),
        None,
    )
    .await;
    let body = body.unwrap();
    let mut options: CreationChallengeResponse =
        serde_json::from_value(body["options"].clone()).unwrap();
    let selection = options.public_key.authenticator_selection.as_mut().unwrap();
    selection.resident_key = Some(ResidentKeyRequirement::Discouraged);
    selection.require_resident_key = false;
    let credential = SoftPasskey::new(true)
        .do_registration(origin(), options)
        .unwrap();

    let (status, _, _) = send(
        &app,
        "POST",
        "/api/passkeys",
        Some(&hanako),
        Some(json!({
            "challenge_id": body["challenge_id"],
            "name": "stolen",
            "credential": credential,
        })),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_blank_or_long_passkey_names() {
    let TestApp { app, .. } = local_app().await;
    let cookie = password_login(&app, "kaito").await;

    for name in ["  ".to_owned(), "a".repeat(65)] {
        let (_, _, body) = send(
            &app,
            "POST",
            "/api/passkeys/registration",
            Some(&cookie),
            None,
        )
        .await;
        let body = body.unwrap();
        let mut options: CreationChallengeResponse =
            serde_json::from_value(body["options"].clone()).unwrap();
        let selection = options.public_key.authenticator_selection.as_mut().unwrap();
        selection.resident_key = Some(ResidentKeyRequirement::Discouraged);
        selection.require_resident_key = false;
        let credential = SoftPasskey::new(true)
            .do_registration(origin(), options)
            .unwrap();

        let (status, _, _) = send(
            &app,
            "POST",
            "/api/passkeys",
            Some(&cookie),
            Some(json!({
                "challenge_id": body["challenge_id"],
                "name": name,
                "credential": credential,
            })),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{name:?}");
    }
}

#[tokio::test]
async fn managing_passkeys_requires_a_session() {
    let TestApp { app, .. } = local_app().await;

    for (method, uri) in [
        ("GET", "/api/passkeys"),
        ("POST", "/api/passkeys/registration"),
        ("DELETE", "/api/passkeys/some-id"),
    ] {
        let (status, _, _) = send(&app, method, uri, None, None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {uri}");
    }
}

#[tokio::test]
async fn passkey_routes_are_absent_when_authentication_is_off() {
    let app = router::auth::create(AuthState::disabled());

    for (method, uri) in [
        ("GET", "/api/passkeys"),
        ("POST", "/api/passkeys/registration"),
        ("POST", "/api/auth/passkey/challenge"),
    ] {
        let (status, _, _) = send(&app, method, uri, None, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{method} {uri}");
    }
    let (_, _, body) = send(&app, "GET", "/api/auth/session", None, None).await;
    assert_eq!(body.unwrap()["passkey"], false);
}

#[tokio::test]
async fn removing_a_user_removes_their_passkeys() {
    let TestApp { app, pool } = local_app().await;
    let cookie = password_login(&app, "kaito").await;
    register(&app, &cookie, &mut SoftPasskey::new(true)).await;

    sqlx::query("DELETE FROM users WHERE username = 'kaito'")
        .execute(&pool)
        .await
        .unwrap();

    let remaining: i64 = sqlx::query_scalar("SELECT count(*) FROM passkeys")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(remaining, 0);
}
