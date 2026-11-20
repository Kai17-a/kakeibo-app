use axum::{http::StatusCode, response::IntoResponse};
use kakeibo_app::{
    auth::{
        database,
        rate_limit::{
            LOCK_SECONDS, LoginRateLimiter, MAX_FAILURES, MAX_TRACKED_KEYS, WINDOW_SECONDS,
        },
    },
    repository::auth::AuthRepository,
    service::auth::{AuthService, MAX_USERNAME_LENGTH},
};
use sqlx::sqlite::SqlitePoolOptions;

const NOW: i64 = 1_000_000;
const USERNAME: &str = "kaito";
const PASSWORD: &str = "correct horse battery staple";

fn fail_times(limiter: &LoginRateLimiter, key: &str, times: u32, now: i64) {
    for _ in 0..times {
        limiter.begin(key, now).expect("attempt allowed").fail(now);
    }
}

#[test]
fn attempts_in_progress_count_toward_the_limit() {
    let limiter = LoginRateLimiter::default();
    let in_progress: Vec<_> = (0..MAX_FAILURES)
        .map(|_| limiter.begin(USERNAME, NOW).expect("attempt allowed"))
        .collect();

    assert!(limiter.begin(USERNAME, NOW).is_none());
    drop(in_progress);
}

#[test]
fn an_abandoned_attempt_frees_its_slot() {
    let limiter = LoginRateLimiter::default();
    let mut in_progress: Vec<_> = (0..MAX_FAILURES)
        .map(|_| limiter.begin(USERNAME, NOW).expect("attempt allowed"))
        .collect();

    drop(in_progress.pop());

    assert!(limiter.begin(USERNAME, NOW).is_some());
}

#[test]
fn repeated_failures_lock_until_the_lock_expires() {
    let limiter = LoginRateLimiter::default();
    fail_times(&limiter, USERNAME, MAX_FAILURES, NOW);

    assert!(limiter.begin(USERNAME, NOW + LOCK_SECONDS - 1).is_none());
    assert!(limiter.begin(USERNAME, NOW + LOCK_SECONDS).is_some());
}

#[test]
fn usernames_are_compared_ignoring_case_and_spaces() {
    let limiter = LoginRateLimiter::default();
    fail_times(&limiter, USERNAME, MAX_FAILURES, NOW);

    assert!(limiter.begin(" KAITO ", NOW).is_none());
}

#[test]
fn a_success_clears_earlier_failures() {
    let limiter = LoginRateLimiter::default();
    fail_times(&limiter, USERNAME, MAX_FAILURES - 1, NOW);

    limiter.begin(USERNAME, NOW).unwrap().succeed();

    fail_times(&limiter, USERNAME, MAX_FAILURES - 1, NOW);
    assert!(limiter.begin(USERNAME, NOW).is_some());
}

#[test]
fn failures_older_than_the_window_are_forgotten() {
    let limiter = LoginRateLimiter::default();
    fail_times(&limiter, USERNAME, MAX_FAILURES - 1, NOW);

    fail_times(&limiter, USERNAME, MAX_FAILURES - 1, NOW + WINDOW_SECONDS);

    assert!(limiter.begin(USERNAME, NOW + WINDOW_SECONDS).is_some());
}

#[test]
fn the_number_of_tracked_usernames_stays_bounded() {
    let limiter = LoginRateLimiter::default();
    for index in 0..MAX_TRACKED_KEYS * 3 {
        if let Some(attempt) = limiter.begin(&format!("user{index}"), NOW) {
            attempt.fail(NOW);
        }
    }

    assert!(limiter.tracked_keys() <= MAX_TRACKED_KEYS);
}

#[test]
fn locked_usernames_are_not_evicted_by_other_failures() {
    let limiter = LoginRateLimiter::default();
    fail_times(&limiter, USERNAME, MAX_FAILURES, NOW);

    for index in 0..MAX_TRACKED_KEYS * 2 {
        if let Some(attempt) = limiter.begin(&format!("user{index}"), NOW + 1) {
            attempt.fail(NOW + 1);
        }
    }

    assert!(limiter.begin(USERNAME, NOW + 2).is_none());
}

#[test]
fn new_usernames_are_refused_while_every_slot_is_locked() {
    let limiter = LoginRateLimiter::default();
    for index in 0..MAX_TRACKED_KEYS {
        fail_times(&limiter, &format!("user{index}"), MAX_FAILURES, NOW);
    }

    // Tracking nothing for the new name would let it be guessed without a limit.
    assert!(limiter.begin(USERNAME, NOW + 1).is_none());
    assert!(limiter.begin(USERNAME, NOW + LOCK_SECONDS).is_some());
}

async fn service() -> AuthService {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    database::migrate(&pool).await.unwrap();
    let service = AuthService::new(AuthRepository::new(pool), 3600);
    service.add_user(USERNAME, PASSWORD).await.unwrap();
    service
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_wrong_passwords_are_checked_at_most_max_failures_times() {
    let service = service().await;
    let attempts: Vec<_> = (0..40)
        .map(|index| {
            let service = service.clone();
            tokio::spawn(async move {
                service
                    .login(USERNAME, &format!("wrong password {index}"), NOW)
                    .await
                    .err()
                    .map(|error| error.into_response().status())
            })
        })
        .collect();

    let mut checked = 0;
    for attempt in attempts {
        match attempt.await.unwrap() {
            Some(StatusCode::UNAUTHORIZED) => checked += 1,
            Some(StatusCode::TOO_MANY_REQUESTS) => {}
            other => panic!("unexpected result: {other:?}"),
        }
    }

    assert_eq!(checked, MAX_FAILURES);
}

#[tokio::test]
async fn overlong_usernames_are_rejected_without_being_tracked() {
    let service = service().await;
    let username = "x".repeat(MAX_USERNAME_LENGTH + 1);

    for _ in 0..MAX_FAILURES + 1 {
        let status = service
            .login(&username, PASSWORD, NOW)
            .await
            .unwrap_err()
            .into_response()
            .status();
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
}
