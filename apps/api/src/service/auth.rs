use crate::{
    auth::{password, rate_limit::LoginRateLimiter, session_cookie},
    repository::auth::AuthRepository,
    utils::error::{AppError, AppResult},
};
use std::sync::Arc;
use tracing::info;

pub const MAX_USERNAME_LENGTH: usize = 64;
/// How often an active session's expiry is pushed forward. Refreshing on every request would
/// write to the database for every API call.
pub const SESSION_REFRESH_INTERVAL_SECONDS: i64 = 60 * 60;

const INVALID_CREDENTIALS: &str = "invalid username or password";

/// A session created by a successful login. `token` is sent to the browser only.
#[derive(Debug)]
pub struct IssuedSession {
    pub token: String,
    pub username: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedUser {
    pub username: String,
    /// True when the expiry moved forward, so the cookie must be sent again.
    pub refreshed: bool,
}

#[derive(Clone)]
pub struct AuthService {
    repository: AuthRepository,
    limiter: Arc<LoginRateLimiter>,
    session_ttl_seconds: i64,
}

impl AuthService {
    pub fn new(repository: AuthRepository, session_ttl_seconds: i64) -> Self {
        Self {
            repository,
            limiter: Arc::new(LoginRateLimiter::default()),
            session_ttl_seconds,
        }
    }

    pub fn session_ttl_seconds(&self) -> i64 {
        self.session_ttl_seconds
    }

    pub async fn login(
        &self,
        username: &str,
        password: &str,
        now: i64,
    ) -> AppResult<IssuedSession> {
        let username = username.trim();
        if username.is_empty() || password.is_empty() {
            return Err(AppError::bad_request("username and password are required"));
        }
        if self.limiter.locked_for(username, now).is_some() {
            return Err(AppError::too_many_requests(
                "too many failed login attempts; try again later",
            ));
        }

        let user = self.repository.find_user(username).await?;
        // Argon2 is deliberately slow, so it runs off the async worker threads.
        let stored_hash = user.as_ref().map(|user| user.password_hash.clone());
        let candidate = password.to_owned();
        let verified = tokio::task::spawn_blocking(move || match stored_hash {
            Some(stored_hash) => password::verify(&candidate, &stored_hash),
            None => {
                password::verify_against_dummy(&candidate);
                false
            }
        })
        .await
        .map_err(|error| AppError::context("Password verification failed", error))?;
        let Some(user) = user.filter(|_| verified) else {
            self.limiter.record_failure(username, now);
            info!(username, "Login failed");
            return Err(AppError::unauthorized(INVALID_CREDENTIALS));
        };

        self.limiter.record_success(username);
        let token = session_cookie::generate_token()?;
        self.repository
            .insert_session(
                &session_cookie::digest(&token),
                &user.id,
                now,
                now.saturating_add(self.session_ttl_seconds),
            )
            .await?;
        info!(username = %user.username, "Login succeeded");
        Ok(IssuedSession {
            token,
            username: user.username,
        })
    }

    pub async fn authenticate(
        &self,
        token: &str,
        now: i64,
    ) -> AppResult<Option<AuthenticatedUser>> {
        let token_hash = session_cookie::digest(token);
        let Some(session) = self.repository.find_session(&token_hash).await? else {
            return Ok(None);
        };
        if session.expires_at <= now {
            self.repository.delete_session(&token_hash).await?;
            return Ok(None);
        }

        let refreshed = now - session.last_seen_at >= SESSION_REFRESH_INTERVAL_SECONDS;
        if refreshed {
            self.repository
                .touch_session(
                    &token_hash,
                    now,
                    now.saturating_add(self.session_ttl_seconds),
                )
                .await?;
        }
        Ok(Some(AuthenticatedUser {
            username: session.username,
            refreshed,
        }))
    }

    pub async fn logout(&self, token: &str) -> AppResult<()> {
        self.repository
            .delete_session(&session_cookie::digest(token))
            .await
    }

    pub async fn delete_expired_sessions(&self, now: i64) -> AppResult<u64> {
        self.repository.delete_expired_sessions(now).await
    }

    pub async fn add_user(&self, username: &str, password: &str) -> AppResult<()> {
        let username = validate_username(username)?;
        password::validate(password)?;
        let hash = password::hash(password)?;
        self.repository.insert_user(username, &hash).await
    }

    /// Changes the password and signs the user out of every session.
    pub async fn set_password(&self, username: &str, password: &str) -> AppResult<()> {
        password::validate(password)?;
        let hash = password::hash(password)?;
        if self
            .repository
            .update_password(username.trim(), &hash)
            .await?
        {
            Ok(())
        } else {
            Err(AppError::not_found("user", username.trim()))
        }
    }

    pub async fn remove_user(&self, username: &str) -> AppResult<()> {
        if self.repository.delete_user(username.trim()).await? {
            Ok(())
        } else {
            Err(AppError::not_found("user", username.trim()))
        }
    }

    pub async fn list_usernames(&self) -> AppResult<Vec<String>> {
        Ok(self
            .repository
            .find_all_users()
            .await?
            .into_iter()
            .map(|user| user.username)
            .collect())
    }

    pub async fn count_users(&self) -> AppResult<i64> {
        self.repository.count_users().await
    }
}

fn validate_username(username: &str) -> AppResult<&str> {
    let username = username.trim();
    if username.is_empty() {
        return Err(AppError::bad_request("username is required"));
    }
    if username.chars().count() > MAX_USERNAME_LENGTH {
        return Err(AppError::bad_request(&format!(
            "username must be at most {MAX_USERNAME_LENGTH} characters"
        )));
    }
    if username
        .chars()
        .any(|character| character.is_whitespace() || character.is_control())
    {
        return Err(AppError::bad_request(
            "username must not contain spaces or control characters",
        ));
    }
    Ok(username)
}
