use crate::{
    auth::AuthState,
    handler::{auth as handler, passkeys},
};
use axum::{
    Router,
    routing::{delete, get, post},
};

pub fn create(state: AuthState) -> Router {
    Router::new()
        .route("/api/auth/session", get(handler::session))
        .route("/api/auth/login", post(handler::login))
        .route("/api/auth/logout", post(handler::logout))
        .route("/api/auth/passkey/challenge", post(passkeys::start_login))
        .route("/api/auth/passkey/login", post(passkeys::login))
        // Managing passkeys needs a session, which the middleware checks for /api/passkeys.
        .route(
            "/api/passkeys",
            get(passkeys::list).post(passkeys::register),
        )
        .route(
            "/api/passkeys/registration",
            post(passkeys::start_registration),
        )
        .route("/api/passkeys/{id}", delete(passkeys::delete))
        .with_state(state)
}
