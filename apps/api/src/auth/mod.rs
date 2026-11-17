//! Optional authentication, enabled with the `KAKEIBO_AUTH_MODE` environment variable.

pub mod challenge_store;
pub mod commands;
pub mod config;
pub mod database;
pub mod middleware;
pub mod password;
pub mod rate_limit;
pub mod session_cookie;

use crate::{
    repository::auth::AuthRepository,
    service::{auth::AuthService, passkeys::PasskeyService},
};
use axum::Router;
use config::{AuthConfig, AuthMode};
use sqlx::SqlitePool;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::warn;

/// Authentication settings and services shared by the auth routes and the middleware.
#[derive(Clone)]
pub struct AuthState {
    pub mode: AuthMode,
    /// Present only when accounts are managed by this application.
    pub service: Option<AuthService>,
    /// Present when the public URL can serve as a WebAuthn relying party.
    pub passkeys: Option<PasskeyService>,
    pub public_origin: String,
    pub secure_cookie: bool,
}

impl AuthState {
    pub fn disabled() -> Self {
        Self {
            mode: AuthMode::None,
            service: None,
            passkeys: None,
            public_origin: String::new(),
            secure_cookie: false,
        }
    }

    /// `pool` must already be migrated with [`database::migrate`].
    pub fn local(config: &AuthConfig, pool: SqlitePool) -> Self {
        let repository = AuthRepository::new(pool);
        let passkeys = PasskeyService::new(repository.clone(), &config.public_origin)
            .inspect_err(|error| warn!(%error, "Passkeys are disabled"))
            .ok();
        Self {
            mode: AuthMode::Local,
            service: Some(
                AuthService::new(repository, config.session_ttl_seconds())
                    .with_min_password_length(config.min_password_length),
            ),
            passkeys,
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
