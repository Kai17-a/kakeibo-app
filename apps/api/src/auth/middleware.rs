use super::{AuthState, now, session_cookie};
use crate::utils::error::AppError;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, Method, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use tracing::warn;

/// The signed-in user, available to handlers as a request extension.
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub user_id: String,
    pub username: String,
}

pub async fn enforce(State(state): State<AuthState>, mut request: Request, next: Next) -> Response {
    let Some(service) = &state.service else {
        return next.run(request).await;
    };
    let path = request.uri().path();

    if path.starts_with("/api/")
        && !is_safe_method(request.method())
        && !is_same_origin(request.headers(), &state.public_origin)
    {
        let origin = request
            .headers()
            .get(header::ORIGIN)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("(none)");
        // Usually a KAKEIBO_PUBLIC_URL that differs from the URL opened in the browser.
        warn!(
            origin,
            expected = %state.public_origin,
            "Rejected a cross-origin request; check KAKEIBO_PUBLIC_URL"
        );
        return AppError::forbidden("cross-origin request rejected").into_response();
    }
    if !requires_session(path) {
        return next.run(request).await;
    }

    let Some(token) = session_cookie::read_token(request.headers()) else {
        return unauthenticated();
    };
    let user = match service.authenticate(&token, now()).await {
        Ok(Some(user)) => user,
        Ok(None) => return unauthenticated(),
        Err(error) => return error.into_response(),
    };

    let refreshed = user.refreshed;
    request.extensions_mut().insert(CurrentUser {
        user_id: user.user_id,
        username: user.username,
    });
    let mut response = next.run(request).await;
    if refreshed {
        response.headers_mut().append(
            header::SET_COOKIE,
            session_cookie::set_cookie(&token, service.session_ttl_seconds(), state.secure_cookie),
        );
    }
    response
}

fn unauthenticated() -> Response {
    AppError::unauthorized("authentication required").into_response()
}

/// The API and its documentation need a session; the auth endpoints, the health check, and the
/// static SPA files do not, so the login screen can load.
fn requires_session(path: &str) -> bool {
    (path.starts_with("/api/") && !path.starts_with("/api/auth/"))
        || path == "/docs"
        || path.starts_with("/docs/")
}

fn is_safe_method(method: &Method) -> bool {
    matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS)
}

/// Rejects state-changing requests sent by another site. Browsers attach `Origin` to such
/// requests; clients that omit it (such as curl) cannot carry the browser's cookie anyway.
fn is_same_origin(headers: &HeaderMap, public_origin: &str) -> bool {
    if let Some(origin) = headers.get(header::ORIGIN) {
        return origin
            .to_str()
            .is_ok_and(|origin| origin.eq_ignore_ascii_case(public_origin));
    }
    headers
        .get("sec-fetch-site")
        .and_then(|value| value.to_str().ok())
        .is_none_or(|site| matches!(site, "same-origin" | "none"))
}
