//! In-memory limit on failed login attempts.
//!
//! Attempts are counted per username, because behind a reverse proxy every request can share
//! one client address. The counters reset when the process restarts, which is acceptable for a
//! single-household deployment.

use std::{collections::HashMap, sync::Mutex};

pub const MAX_FAILURES: u32 = 5;
pub const WINDOW_SECONDS: i64 = 15 * 60;
pub const LOCK_SECONDS: i64 = 15 * 60;
/// Bounds memory when someone tries many different usernames.
const MAX_TRACKED_KEYS: usize = 1024;

#[derive(Debug, Clone, Copy)]
struct Attempts {
    failures: u32,
    first_failure_at: i64,
    locked_until: Option<i64>,
}

#[derive(Debug, Default)]
pub struct LoginRateLimiter {
    attempts: Mutex<HashMap<String, Attempts>>,
}

impl LoginRateLimiter {
    /// Returns the seconds until the key is unlocked, or `None` when a login may be attempted.
    pub fn locked_for(&self, key: &str, now: i64) -> Option<i64> {
        let attempts = self.lock();
        attempts
            .get(&normalize(key))
            .and_then(|entry| entry.locked_until)
            .filter(|until| *until > now)
            .map(|until| until - now)
    }

    pub fn record_failure(&self, key: &str, now: i64) {
        let mut attempts = self.lock();
        if attempts.len() >= MAX_TRACKED_KEYS {
            attempts.retain(|_, entry| is_relevant(entry, now));
        }
        let entry = attempts.entry(normalize(key)).or_insert(Attempts {
            failures: 0,
            first_failure_at: now,
            locked_until: None,
        });
        if !is_relevant(entry, now) {
            *entry = Attempts {
                failures: 0,
                first_failure_at: now,
                locked_until: None,
            };
        }
        entry.failures += 1;
        if entry.failures >= MAX_FAILURES {
            entry.locked_until = Some(now + LOCK_SECONDS);
        }
    }

    pub fn record_success(&self, key: &str) {
        self.lock().remove(&normalize(key));
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, Attempts>> {
        self.attempts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

fn normalize(key: &str) -> String {
    key.trim().to_lowercase()
}

fn is_relevant(entry: &Attempts, now: i64) -> bool {
    match entry.locked_until {
        Some(until) => until > now,
        None => now - entry.first_failure_at < WINDOW_SECONDS,
    }
}
