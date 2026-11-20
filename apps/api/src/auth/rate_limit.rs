//! In-memory limit on failed login attempts.
//!
//! Attempts are counted per username, because behind a reverse proxy every request can share
//! one client address. The counters reset when the process restarts, which is acceptable for a
//! single-household deployment.
//!
//! An attempt reserves its slot with [`LoginRateLimiter::begin`] before the password is checked,
//! so requests sent at the same time cannot get past the limit together.

use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard, PoisonError},
};

pub const MAX_FAILURES: u32 = 5;
pub const WINDOW_SECONDS: i64 = 15 * 60;
pub const LOCK_SECONDS: i64 = 15 * 60;
/// Bounds memory when someone tries many different usernames.
pub const MAX_TRACKED_KEYS: usize = 1024;

#[derive(Debug, Clone, Copy)]
struct Attempts {
    failures: u32,
    /// Attempts that have started but not finished yet.
    in_progress: u32,
    first_failure_at: i64,
    locked_until: Option<i64>,
}

impl Attempts {
    fn new(now: i64) -> Self {
        Self {
            failures: 0,
            in_progress: 0,
            first_failure_at: now,
            locked_until: None,
        }
    }

    fn is_locked(&self, now: i64) -> bool {
        self.locked_until.is_some_and(|until| until > now)
    }

    /// Forgets failures whose lock or counting window has ended, keeping attempts in progress.
    fn expire(&mut self, now: i64) {
        let stale = match self.locked_until {
            Some(until) => until <= now,
            None => now - self.first_failure_at >= WINDOW_SECONDS,
        };
        if stale {
            self.failures = 0;
            self.first_failure_at = now;
            self.locked_until = None;
        }
    }

    /// Whether the entry still affects future attempts.
    fn is_relevant(&self, now: i64) -> bool {
        self.in_progress > 0
            || match self.locked_until {
                Some(until) => until > now,
                None => self.failures > 0 && now - self.first_failure_at < WINDOW_SECONDS,
            }
    }
}

#[derive(Debug, Default)]
pub struct LoginRateLimiter {
    attempts: Mutex<HashMap<String, Attempts>>,
}

impl LoginRateLimiter {
    /// Reserves a login attempt for `key`. Returns `None` when the key is locked, when the
    /// attempts already in progress would reach the limit, or when no more keys can be tracked.
    pub fn begin(&self, key: &str, now: i64) -> Option<Attempt<'_>> {
        let key = normalize(key);
        let mut attempts = self.lock();
        if !attempts.contains_key(&key) && !make_room(&mut attempts, now) {
            return None;
        }

        let entry = attempts
            .entry(key.clone())
            .or_insert_with(|| Attempts::new(now));
        entry.expire(now);
        if entry.is_locked(now) || entry.failures + entry.in_progress >= MAX_FAILURES {
            return None;
        }
        entry.in_progress += 1;
        Some(Attempt {
            limiter: self,
            key: Some(key),
        })
    }

    /// Returns the seconds until the key is unlocked, or `None` when it is not locked.
    pub fn locked_for(&self, key: &str, now: i64) -> Option<i64> {
        self.lock()
            .get(&normalize(key))
            .and_then(|entry| entry.locked_until)
            .filter(|until| *until > now)
            .map(|until| until - now)
    }

    /// The number of usernames currently remembered.
    pub fn tracked_keys(&self) -> usize {
        self.lock().len()
    }

    fn finish(&self, key: &str, outcome: Outcome) {
        let mut attempts = self.lock();
        let Some(entry) = attempts.get_mut(key) else {
            return;
        };
        entry.in_progress = entry.in_progress.saturating_sub(1);
        match outcome {
            Outcome::Succeeded => {
                attempts.remove(key);
            }
            Outcome::Failed(now) => {
                if entry.failures == 0 {
                    entry.first_failure_at = now;
                }
                entry.failures += 1;
                if entry.failures >= MAX_FAILURES {
                    entry.locked_until = Some(now + LOCK_SECONDS);
                }
            }
            Outcome::Abandoned => {
                if entry.failures == 0 && entry.in_progress == 0 {
                    attempts.remove(key);
                }
            }
        }
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<String, Attempts>> {
        self.attempts.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// A reserved login attempt. Report the result with [`Attempt::fail`] or [`Attempt::succeed`];
/// dropping it without a result (for example when the client disconnects) frees the slot.
#[must_use = "report the result of the attempt"]
pub struct Attempt<'a> {
    limiter: &'a LoginRateLimiter,
    key: Option<String>,
}

impl Attempt<'_> {
    pub fn fail(mut self, now: i64) {
        if let Some(key) = self.key.take() {
            self.limiter.finish(&key, Outcome::Failed(now));
        }
    }

    pub fn succeed(mut self) {
        if let Some(key) = self.key.take() {
            self.limiter.finish(&key, Outcome::Succeeded);
        }
    }
}

impl Drop for Attempt<'_> {
    fn drop(&mut self) {
        if let Some(key) = self.key.take() {
            self.limiter.finish(&key, Outcome::Abandoned);
        }
    }
}

enum Outcome {
    Succeeded,
    Failed(i64),
    Abandoned,
}

/// Makes space for one more key. Locked keys and attempts in progress are never dropped, so
/// filling the table cannot unlock a username; when nothing can be dropped, it reports false
/// and the new key is refused instead of being left untracked.
fn make_room(attempts: &mut HashMap<String, Attempts>, now: i64) -> bool {
    if attempts.len() < MAX_TRACKED_KEYS {
        return true;
    }
    attempts.retain(|_, entry| entry.is_relevant(now));
    if attempts.len() < MAX_TRACKED_KEYS {
        return true;
    }
    let oldest = attempts
        .iter()
        .filter(|(_, entry)| !entry.is_locked(now) && entry.in_progress == 0)
        .min_by_key(|(_, entry)| entry.first_failure_at)
        .map(|(key, _)| key.clone());
    match oldest {
        Some(key) => {
            attempts.remove(&key);
            true
        }
        None => false,
    }
}

fn normalize(key: &str) -> String {
    key.trim().to_lowercase()
}
