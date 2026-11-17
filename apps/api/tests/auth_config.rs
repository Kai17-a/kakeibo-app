use kakeibo_app::auth::config::{AuthConfig, AuthMode};
use std::{collections::HashMap, path::PathBuf, time::Duration};

fn parse(vars: &[(&str, &str)]) -> Result<AuthConfig, String> {
    let vars: HashMap<String, String> = vars
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect();
    AuthConfig::from_lookup(|key| vars.get(key).cloned()).map_err(|error| error.to_string())
}

#[test]
fn defaults_to_no_authentication() {
    let config = parse(&[]).unwrap();

    assert_eq!(config.mode, AuthMode::None);
}

#[test]
fn accepts_explicit_none_mode() {
    let config = parse(&[("KAKEIBO_AUTH_MODE", "none")]).unwrap();

    assert_eq!(config.mode, AuthMode::None);
}

#[test]
fn local_mode_uses_defaults() {
    let config = parse(&[
        ("KAKEIBO_AUTH_MODE", "local"),
        ("KAKEIBO_PUBLIC_URL", "https://kakeibo.example.com/"),
    ])
    .unwrap();

    assert_eq!(config.mode, AuthMode::Local);
    assert_eq!(config.public_origin, "https://kakeibo.example.com");
    assert!(config.secure_cookie);
    assert_eq!(config.session_ttl, Duration::from_secs(30 * 24 * 60 * 60));
    assert_eq!(config.database_path, PathBuf::from("auth.db"));
}

#[test]
fn mode_is_case_insensitive() {
    let config = parse(&[
        ("KAKEIBO_AUTH_MODE", " Local "),
        ("KAKEIBO_PUBLIC_URL", "http://localhost:8000"),
    ])
    .unwrap();

    assert_eq!(config.mode, AuthMode::Local);
}

#[test]
fn http_public_url_disables_secure_cookie() {
    let config = parse(&[
        ("KAKEIBO_AUTH_MODE", "local"),
        ("KAKEIBO_PUBLIC_URL", "http://localhost:8000"),
    ])
    .unwrap();

    assert!(!config.secure_cookie);
    assert_eq!(config.public_origin, "http://localhost:8000");
}

#[test]
fn public_url_path_is_not_part_of_origin() {
    let config = parse(&[
        ("KAKEIBO_AUTH_MODE", "local"),
        ("KAKEIBO_PUBLIC_URL", "https://Example.com:8443/kakeibo"),
    ])
    .unwrap();

    assert_eq!(config.public_origin, "https://example.com:8443");
}

#[test]
fn local_mode_requires_public_url() {
    let error = parse(&[("KAKEIBO_AUTH_MODE", "local")]).unwrap_err();

    assert!(error.contains("KAKEIBO_PUBLIC_URL"), "{error}");
}

#[test]
fn rejects_public_url_without_http_scheme() {
    let error = parse(&[
        ("KAKEIBO_AUTH_MODE", "local"),
        ("KAKEIBO_PUBLIC_URL", "kakeibo.example.com"),
    ])
    .unwrap_err();

    assert!(error.contains("KAKEIBO_PUBLIC_URL"), "{error}");
}

#[test]
fn rejects_unknown_mode() {
    let error = parse(&[("KAKEIBO_AUTH_MODE", "oidc")]).unwrap_err();

    assert!(error.contains("KAKEIBO_AUTH_MODE"), "{error}");
}

#[test]
fn parses_session_ttl_units() {
    for (value, seconds) in [
        ("90", 90),
        ("45m", 45 * 60),
        ("12h", 12 * 60 * 60),
        ("7d", 7 * 24 * 60 * 60),
    ] {
        let config = parse(&[
            ("KAKEIBO_AUTH_MODE", "local"),
            ("KAKEIBO_PUBLIC_URL", "https://kakeibo.example.com"),
            ("KAKEIBO_AUTH_SESSION_TTL", value),
        ])
        .unwrap();

        assert_eq!(config.session_ttl, Duration::from_secs(seconds), "{value}");
    }
}

#[test]
fn rejects_invalid_session_ttl() {
    for value in ["", "0", "-1h", "1w", "abc"] {
        let error = parse(&[
            ("KAKEIBO_AUTH_MODE", "local"),
            ("KAKEIBO_PUBLIC_URL", "https://kakeibo.example.com"),
            ("KAKEIBO_AUTH_SESSION_TTL", value),
        ])
        .unwrap_err();

        assert!(
            error.contains("KAKEIBO_AUTH_SESSION_TTL"),
            "{value}: {error}"
        );
    }
}

#[test]
fn accepts_custom_database_path() {
    let config = parse(&[
        ("KAKEIBO_AUTH_MODE", "local"),
        ("KAKEIBO_PUBLIC_URL", "https://kakeibo.example.com"),
        ("KAKEIBO_AUTH_DATABASE", "/data/custom-auth.db"),
    ])
    .unwrap();

    assert_eq!(config.database_path, PathBuf::from("/data/custom-auth.db"));
}
