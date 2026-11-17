use crate::{auth::AuthState, handler::auth as handler};
use axum::{
    Router,
    routing::{get, post},
};

pub fn create(state: AuthState) -> Router {
    Router::new()
        .route("/api/auth/session", get(handler::session))
        .route("/api/auth/login", post(handler::login))
        .route("/api/auth/logout", post(handler::logout))
        .with_state(state)
}
