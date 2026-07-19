use axum::{Router, routing::get};

use crate::handler::health;

pub fn create() -> Router {
    Router::new().route("/health", get(health::get))
}
