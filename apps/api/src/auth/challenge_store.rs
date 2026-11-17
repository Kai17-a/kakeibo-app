//! Short-lived WebAuthn ceremony state kept in memory.
//!
//! webauthn-rs requires the registration and authentication state to stay on the server, so it
//! is stored here under a random ID that the browser sends back to finish the ceremony.

use super::session_cookie;
use crate::utils::error::{AppError, AppResult};
use std::{collections::HashMap, sync::Mutex};

pub const CHALLENGE_TTL_SECONDS: i64 = 5 * 60;
const MAX_PENDING: usize = 256;

pub struct ChallengeStore<T> {
    pending: Mutex<HashMap<String, (i64, T)>>,
}

impl<T> Default for ChallengeStore<T> {
    fn default() -> Self {
        Self {
            pending: Mutex::new(HashMap::new()),
        }
    }
}

impl<T> ChallengeStore<T> {
    /// Stores `state` and returns the ID that retrieves it.
    pub fn insert(&self, state: T, now: i64) -> AppResult<String> {
        let id = session_cookie::generate_token()?;
        let mut pending = self.lock();
        if pending.len() >= MAX_PENDING {
            pending.retain(|_, (expires_at, _)| *expires_at > now);
        }
        if pending.len() >= MAX_PENDING {
            return Err(AppError::too_many_requests(
                "too many passkey requests in progress; try again later",
            ));
        }
        pending.insert(id.clone(), (now + CHALLENGE_TTL_SECONDS, state));
        Ok(id)
    }

    /// Removes and returns the state, so each challenge can be answered only once.
    pub fn take(&self, id: &str, now: i64) -> Option<T> {
        self.lock()
            .remove(id)
            .filter(|(expires_at, _)| *expires_at > now)
            .map(|(_, state)| state)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, (i64, T)>> {
        self.pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
