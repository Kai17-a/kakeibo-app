//! Optional authentication, enabled with the `KAKEIBO_AUTH_MODE` environment variable.

pub mod commands;
pub mod config;
pub mod database;
pub mod middleware;
pub mod password;
pub mod rate_limit;
pub mod session_cookie;

use crate::{repository::auth::AuthRepository, service::auth::AuthService};
use axum::Router;
use config::{AuthConfig, AuthMode};
use sqlx::SqlitePool;
use std::time::{SystemTime, UNIX_EPOCH};

/// Authentication settings and services shared by the auth routes and the middleware.
#[derive(Clone)]
pub struct AuthState {
    pub mode: AuthMode,
    /// Present only when accounts are managed by this application.
    pub service: Option<AuthService>,
    pub public_origin: String,
    pub secure_cookie: bool,
}

impl AuthState {
    pub fn disabled() -> Self {
        Self {
            mode: AuthMode::None,
            service: None,
            public_origin: String::new(),
            secure_cookie: false,
        }
    }

    /// `pool` must already be migrated with [`database::migrate`].
    pub fn local(config: &AuthConfig, pool: SqlitePool) -> Self {
        Self {
            mode: AuthMode::Local,
            service: Some(AuthService::new(
                AuthRepository::new(pool),
                config.session_ttl_seconds(),
            )),
            public_origin: config.public_origin.clone(),
            secure_cookie: config.secure_cookie,
        }
    }
}

/// Requires a session for the API and the API documentation when authentication is on.
/// Call it after every route and the fallback are added, so the check covers all of them.
pub fn protect(router: Router, state: AuthState) -> Router {
    if state.service.is_none() {
        return router;
    }
    router.layer(axum::middleware::from_fn_with_state(
        state,
        middleware::enforce,
    ))
}

/// Current time as Unix epoch seconds.
pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            i64::try_from(duration.as_secs()).unwrap_or(i64::MAX)
        })
}
