//! Password hashing with Argon2id.
//!
//! The default Argon2 parameters (19 MiB of memory, 2 iterations, 1 lane) match the OWASP
//! Password Storage Cheat Sheet recommendation. The salt is generated per password and stored
//! inside the PHC-format hash string.

use crate::utils::error::{AppError, AppResult};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use std::sync::OnceLock;

/// NIST SP 800-63B asks for at least 15 characters when a password is the only factor.
pub const MIN_PASSWORD_LENGTH: usize = 15;
pub const MAX_PASSWORD_LENGTH: usize = 256;

pub fn validate(password: &str) -> AppResult<()> {
    let length = password.chars().count();
    if length < MIN_PASSWORD_LENGTH {
        return Err(AppError::bad_request(&format!(
            "password must be at least {MIN_PASSWORD_LENGTH} characters"
        )));
    }
    if length > MAX_PASSWORD_LENGTH {
        return Err(AppError::bad_request(&format!(
            "password must be at most {MAX_PASSWORD_LENGTH} characters"
        )));
    }
    Ok(())
}

pub fn hash(password: &str) -> AppResult<String> {
    let mut salt = [0_u8; 16];
    getrandom::fill(&mut salt)
        .map_err(|error| AppError::context("Failed to generate a password salt", error))?;
    let salt = SaltString::encode_b64(&salt)
        .map_err(|error| AppError::context("Failed to encode a password salt", error))?;
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| AppError::context("Failed to hash a password", error))
}

/// Returns whether `password` matches `stored`. A malformed stored hash never matches.
pub fn verify(password: &str, stored: &str) -> bool {
    PasswordHash::new(stored).is_ok_and(|parsed| {
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    })
}

/// Spends the same time as [`verify`] for a username that does not exist, so response timing
/// does not reveal which usernames are registered.
pub fn verify_against_dummy(password: &str) {
    static DUMMY: OnceLock<Option<String>> = OnceLock::new();
    if let Some(dummy) = DUMMY.get_or_init(|| hash("kakeibo-dummy-password").ok()) {
        let _ = verify(password, dummy);
    }
}
