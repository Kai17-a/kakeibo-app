use axum::http::StatusCode;
use axum::response::IntoResponse;
use kakeibo_app::{
    auth::{database, password::DEFAULT_MIN_PASSWORD_LENGTH},
    repository::auth::AuthRepository,
    service::auth::AuthService,
};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

const PASSWORD: &str = "correct horse battery staple";

async fn service() -> (AuthService, SqlitePool) {
    service_with_min_length(DEFAULT_MIN_PASSWORD_LENGTH).await
}

async fn service_with_min_length(min_password_length: usize) -> (AuthService, SqlitePool) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    database::migrate(&pool).await.unwrap();
    (
        AuthService::new(AuthRepository::new(pool.clone()), 3600)
            .with_min_password_length(min_password_length),
        pool,
    )
}

fn status(error: kakeibo_app::utils::error::AppError) -> StatusCode {
    error.into_response().status()
}

#[tokio::test]
async fn adds_and_lists_users() {
    let (service, _) = service().await;

    service.add_user("kaito", PASSWORD).await.unwrap();
    service.add_user("hanako", PASSWORD).await.unwrap();

    assert_eq!(service.list_usernames().await.unwrap(), ["hanako", "kaito"]);
    assert_eq!(service.count_users().await.unwrap(), 2);
}

#[tokio::test]
async fn stores_argon2id_hashes_only() {
    let (service, pool) = service().await;
    service.add_user("kaito", PASSWORD).await.unwrap();

    let stored: String = sqlx::query_scalar("SELECT password_hash FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert!(stored.starts_with("$argon2id$"), "{stored}");
    assert!(!stored.contains(PASSWORD));
}

#[tokio::test]
async fn rejects_duplicate_usernames_ignoring_case() {
    let (service, _) = service().await;
    service.add_user("kaito", PASSWORD).await.unwrap();

    let error = service.add_user("Kaito", PASSWORD).await.unwrap_err();

    assert_eq!(status(error), StatusCode::CONFLICT);
}

#[tokio::test]
async fn rejects_short_passwords() {
    let (service, _) = service().await;
    let short = "a".repeat(DEFAULT_MIN_PASSWORD_LENGTH - 1);

    let error = service.add_user("kaito", &short).await.unwrap_err();

    assert_eq!(status(error), StatusCode::BAD_REQUEST);
    assert!(
        service
            .add_user("kaito", &"a".repeat(DEFAULT_MIN_PASSWORD_LENGTH))
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn rejects_invalid_usernames() {
    let (service, _) = service().await;

    for username in ["", "  ", "with space", "tab\tname", &"a".repeat(65)] {
        let error = service.add_user(username, PASSWORD).await.unwrap_err();
        assert_eq!(status(error), StatusCode::BAD_REQUEST, "{username:?}");
    }
}

#[tokio::test]
async fn changing_the_password_signs_the_user_out() {
    let (service, _) = service().await;
    service.add_user("kaito", PASSWORD).await.unwrap();
    let session = service.login("kaito", PASSWORD, 100).await.unwrap();

    service
        .set_password("kaito", "another long passphrase")
        .await
        .unwrap();

    assert!(
        service
            .authenticate(&session.token, 101)
            .await
            .unwrap()
            .is_none()
    );
    assert!(service.login("kaito", PASSWORD, 102).await.is_err());
    assert!(
        service
            .login("kaito", "another long passphrase", 103)
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn removing_a_user_deletes_their_sessions() {
    let (service, pool) = service().await;
    service.add_user("kaito", PASSWORD).await.unwrap();
    let session = service.login("kaito", PASSWORD, 100).await.unwrap();

    service.remove_user("kaito").await.unwrap();

    assert!(
        service
            .authenticate(&session.token, 101)
            .await
            .unwrap()
            .is_none()
    );
    let sessions: i64 = sqlx::query_scalar("SELECT count(*) FROM sessions")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(sessions, 0);
}

#[tokio::test]
async fn missing_users_are_reported() {
    let (service, _) = service().await;

    assert_eq!(
        status(service.set_password("nobody", PASSWORD).await.unwrap_err()),
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        status(service.remove_user("nobody").await.unwrap_err()),
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn deletes_only_expired_sessions() {
    let (service, _) = service().await;
    service.add_user("kaito", PASSWORD).await.unwrap();
    let old = service.login("kaito", PASSWORD, 0).await.unwrap();
    let fresh = service.login("kaito", PASSWORD, 5000).await.unwrap();

    assert_eq!(service.delete_expired_sessions(4000).await.unwrap(), 1);

    assert!(
        service
            .authenticate(&old.token, 4000)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        service
            .authenticate(&fresh.token, 4000)
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn uses_the_configured_minimum_password_length() {
    let (service, _) = service_with_min_length(20).await;

    let error = service
        .add_user("kaito", &"a".repeat(19))
        .await
        .unwrap_err();
    assert_eq!(status(error), StatusCode::BAD_REQUEST);
    service.add_user("kaito", &"a".repeat(20)).await.unwrap();

    let error = service
        .set_password("kaito", &"b".repeat(19))
        .await
        .unwrap_err();
    assert_eq!(status(error), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn a_minimum_of_one_accepts_any_non_empty_password() {
    let (service, _) = service_with_min_length(1).await;

    service.add_user("kaito", "a").await.unwrap();
    assert_eq!(
        status(service.add_user("hanako", "").await.unwrap_err()),
        StatusCode::BAD_REQUEST
    );
}
