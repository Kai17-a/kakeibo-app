use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct SessionResponse {
    /// Authentication mode configured on the server: `none` or `local`.
    pub mode: &'static str,
    /// Always true when authentication is off.
    pub authenticated: bool,
    pub user: Option<SessionUser>,
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
