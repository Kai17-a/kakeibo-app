//! Authentication settings read from environment variables.
//!
//! Authentication is off unless `KAKEIBO_AUTH_MODE` selects a mode, so existing deployments keep
//! working without any change.

use super::password;
use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
    time::Duration,
};

pub const MODE_VAR: &str = "KAKEIBO_AUTH_MODE";
pub const PUBLIC_URL_VAR: &str = "KAKEIBO_PUBLIC_URL";
pub const SESSION_TTL_VAR: &str = "KAKEIBO_AUTH_SESSION_TTL";
pub const DATABASE_VAR: &str = "KAKEIBO_AUTH_DATABASE";
pub const MIN_PASSWORD_LENGTH_VAR: &str = "KAKEIBO_AUTH_MIN_PASSWORD_LENGTH";

const DEFAULT_SESSION_TTL: Duration = Duration::from_secs(30 * 24 * 60 * 60);
const DEFAULT_DATABASE_PATH: &str = "auth.db";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMode {
    /// No authentication: every request is allowed.
    None,
    /// Username and password accounts stored by this application.
    Local,
}

impl AuthMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Local => "local",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub mode: AuthMode,
    /// `scheme://host[:port]` of the public URL, compared with the `Origin` request header.
    /// Empty when authentication is off.
    pub public_origin: String,
    pub secure_cookie: bool,
    pub session_ttl: Duration,
    /// SQLite file for accounts and sessions. It is kept apart from the household database so
    /// that restoring a backup never replaces credentials.
    pub database_path: PathBuf,
    /// Shortest password accepted when adding a user or changing a password.
    pub min_password_length: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ConfigError(String);

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ConfigError {}

impl AuthConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|key| std::env::var(key).ok())
    }

    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let mode = match lookup(MODE_VAR) {
            None => AuthMode::None,
            Some(value) => parse_mode(&value)?,
        };
        let database_path = database_path(&lookup);
        let min_password_length = min_password_length(&lookup)?;

        if mode == AuthMode::None {
            return Ok(Self {
                mode,
                public_origin: String::new(),
                secure_cookie: false,
                session_ttl: DEFAULT_SESSION_TTL,
                database_path,
                min_password_length,
            });
        }

        let public_url = lookup(PUBLIC_URL_VAR).ok_or_else(|| {
            ConfigError(format!(
                "{PUBLIC_URL_VAR} is required when {MODE_VAR}={}",
                mode.as_str()
            ))
        })?;
        let (public_origin, secure_cookie) = parse_public_url(&public_url)?;
        let session_ttl = match lookup(SESSION_TTL_VAR) {
            None => DEFAULT_SESSION_TTL,
            Some(value) => parse_duration(&value).ok_or_else(|| {
                ConfigError(format!(
                    "{SESSION_TTL_VAR} must be a positive number of seconds or use the m, h, or d suffix"
                ))
            })?,
        };

        Ok(Self {
            mode,
            public_origin,
            secure_cookie,
            session_ttl,
            database_path,
            min_password_length,
        })
    }

    pub fn session_ttl_seconds(&self) -> i64 {
        i64::try_from(self.session_ttl.as_secs()).unwrap_or(i64::MAX)
    }
}

/// The authentication database path, which the account commands need without the rest of the
/// settings.
pub fn database_path(lookup: impl Fn(&str) -> Option<String>) -> PathBuf {
    lookup(DATABASE_VAR)
        .filter(|value| !value.trim().is_empty())
        .map_or_else(|| PathBuf::from(DEFAULT_DATABASE_PATH), PathBuf::from)
}

/// The minimum password length, which the account commands also need.
pub fn min_password_length(lookup: impl Fn(&str) -> Option<String>) -> Result<usize, ConfigError> {
    let Some(value) = lookup(MIN_PASSWORD_LENGTH_VAR) else {
        return Ok(password::DEFAULT_MIN_PASSWORD_LENGTH);
    };
    value
        .trim()
        .parse::<usize>()
        .ok()
        .filter(|length| (1..=password::MAX_PASSWORD_LENGTH).contains(length))
        .ok_or_else(|| {
            ConfigError(format!(
                "{MIN_PASSWORD_LENGTH_VAR} must be a number from 1 to {}",
                password::MAX_PASSWORD_LENGTH
            ))
        })
}

fn parse_mode(value: &str) -> Result<AuthMode, ConfigError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "" | "none" => Ok(AuthMode::None),
        "local" => Ok(AuthMode::Local),
        _ => Err(ConfigError(format!(
            "{MODE_VAR} must be one of: none, local"
        ))),
    }
}

/// Returns the origin of `value` and whether it uses HTTPS.
fn parse_public_url(value: &str) -> Result<(String, bool), ConfigError> {
    let invalid = || {
        ConfigError(format!(
            "{PUBLIC_URL_VAR} must be an absolute http or https URL such as https://kakeibo.example.com"
        ))
    };
    let value = value.trim();
    let (scheme, rest) = value.split_once("://").ok_or_else(invalid)?;
    let scheme = scheme.to_ascii_lowercase();
    let secure = match scheme.as_str() {
        "https" => true,
        "http" => false,
        _ => return Err(invalid()),
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    if authority.is_empty() || authority.contains('@') || authority.contains(char::is_whitespace) {
        return Err(invalid());
    }

    Ok((
        format!("{scheme}://{}", authority.to_ascii_lowercase()),
        secure,
    ))
}

fn parse_duration(value: &str) -> Option<Duration> {
    let value = value.trim();
    let (number, unit) = match value.char_indices().last()? {
        (index, 'm') => (&value[..index], 60),
        (index, 'h') => (&value[..index], 60 * 60),
        (index, 'd') => (&value[..index], 24 * 60 * 60),
        _ => (value, 1),
    };
    let number: u64 = number.parse().ok()?;
    let seconds = number.checked_mul(unit)?;
    (seconds > 0).then(|| Duration::from_secs(seconds))
}
