use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct SessionResponse {
    /// Authentication mode configured on the server: `none` or `local`.
    pub mode: &'static str,
    /// Always true when authentication is off.
    pub authenticated: bool,
    pub user: Option<SessionUser>,
    /// Whether passkey login is available (local mode with a public URL usable for WebAuthn).
    pub passkey: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SessionUser {
    pub username: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PasskeySummary {
    /// WebAuthn credential ID (base64url).
    pub id: String,
    pub name: String,
    /// Unix epoch seconds.
    pub created_at: i64,
    /// Unix epoch seconds of the last login, or null when never used.
    pub last_used_at: Option<i64>,
}

/// Options for `navigator.credentials.create` or `get`, and the ID to send back with the result.
#[derive(Debug, Serialize, ToSchema)]
pub struct PasskeyChallenge {
    pub challenge_id: String,
    #[schema(value_type = Object)]
    pub options: serde_json::Value,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PasskeyRegistrationRequest {
    pub challenge_id: String,
    pub name: String,
    /// The `PublicKeyCredential` from `navigator.credentials.create`, encoded as JSON.
    #[schema(value_type = Object)]
    pub credential: webauthn_rs::prelude::RegisterPublicKeyCredential,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PasskeyLoginRequest {
    pub challenge_id: String,
    /// The `PublicKeyCredential` from `navigator.credentials.get`, encoded as JSON.
    #[schema(value_type = Object)]
    pub credential: webauthn_rs::prelude::PublicKeyCredential,
}
