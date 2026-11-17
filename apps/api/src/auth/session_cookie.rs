//! Session tokens and the cookie that carries them.

use crate::utils::error::{AppError, AppResult};
use axum::http::{HeaderMap, HeaderValue, header};
use sha2::{Digest, Sha256};
use std::fmt::Write;

pub const COOKIE_NAME: &str = "kakeibo_session";
const TOKEN_BYTES: usize = 32;

/// Generates a new random session token as 64 hexadecimal characters.
pub fn generate_token() -> AppResult<String> {
    let mut bytes = [0_u8; TOKEN_BYTES];
    getrandom::fill(&mut bytes)
        .map_err(|error| AppError::context("Failed to generate a session token", error))?;
    Ok(to_hex(&bytes))
}

/// The value stored in the database for `token`.
pub fn digest(token: &str) -> String {
    to_hex(&Sha256::digest(token.as_bytes()))
}

/// Reads the session token from the `Cookie` request headers, ignoring malformed values.
pub fn read_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(name, _)| *name == COOKIE_NAME)
        .map(|(_, value)| value.trim())
        .filter(|value| is_well_formed(value))
        .map(str::to_owned)
}

pub fn set_cookie(token: &str, max_age_seconds: i64, secure: bool) -> HeaderValue {
    build(token, max_age_seconds, secure)
}

pub fn clear_cookie(secure: bool) -> HeaderValue {
    build("", 0, secure)
}

fn build(value: &str, max_age_seconds: i64, secure: bool) -> HeaderValue {
    let mut cookie =
        format!("{COOKIE_NAME}={value}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age_seconds}");
    if secure {
        cookie.push_str("; Secure");
    }
    HeaderValue::from_str(&cookie).expect("session cookie contains only visible ASCII")
}

fn is_well_formed(token: &str) -> bool {
    token.len() == TOKEN_BYTES * 2 && token.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        })
}
