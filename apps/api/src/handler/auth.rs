use crate::{
    auth::{AuthState, now, session_cookie},
    model::auth::{LoginRequest, SessionResponse, SessionUser},
    service::auth::AuthService,
    utils::error::{AppError, AppResult},
};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};

#[utoipa::path(get,path="/api/auth/session",responses((status=200,body=SessionResponse)))]
pub async fn session(State(state): State<AuthState>, headers: HeaderMap) -> AppResult<Response> {
    let Some(service) = &state.service else {
        return Ok(Json(SessionResponse {
            mode: state.mode.as_str(),
            authenticated: true,
            user: None,
            passkey: false,
        })
        .into_response());
    };

    let user = match session_cookie::read_token(&headers) {
        Some(token) => service
            .authenticate(&token, now())
            .await?
            .map(|user| (token, user)),
        None => None,
    };
    let body = SessionResponse {
        mode: state.mode.as_str(),
        authenticated: user.is_some(),
        user: user.as_ref().map(|(_, user)| SessionUser {
            username: user.username.clone(),
        }),
        passkey: state.passkeys.is_some(),
    };
    let mut response = Json(body).into_response();
    if let Some((token, user)) = user
        && user.refreshed
    {
        response.headers_mut().append(
            header::SET_COOKIE,
            session_cookie::set_cookie(&token, service.session_ttl_seconds(), state.secure_cookie),
        );
    }
    Ok(response)
}

#[utoipa::path(post,path="/api/auth/login",request_body=LoginRequest,responses((status=204),(status=400),(status=401),(status=404),(status=429)))]
pub async fn login(
    State(state): State<AuthState>,
    headers: HeaderMap,
    Json(request): Json<LoginRequest>,
) -> AppResult<Response> {
    let service = state.service.as_ref().ok_or_else(disabled)?;
    let session = service
        .login(&request.username, &request.password, now())
        .await?;
    signed_in(&state, service, &headers, &session.token).await
}

/// Ends any session the browser still holds and sends the cookie for the new one.
pub(crate) async fn signed_in(
    state: &AuthState,
    service: &AuthService,
    headers: &HeaderMap,
    token: &str,
) -> AppResult<Response> {
    if let Some(previous) = session_cookie::read_token(headers) {
        service.logout(&previous).await?;
    }
    Ok((
        StatusCode::NO_CONTENT,
        [(
            header::SET_COOKIE,
            session_cookie::set_cookie(token, service.session_ttl_seconds(), state.secure_cookie),
        )],
    )
        .into_response())
}

#[utoipa::path(post,path="/api/auth/logout",responses((status=204),(status=404)))]
pub async fn logout(State(state): State<AuthState>, headers: HeaderMap) -> AppResult<Response> {
    let service = state.service.as_ref().ok_or_else(disabled)?;
    if let Some(token) = session_cookie::read_token(&headers) {
        service.logout(&token).await?;
    }
    Ok((
        StatusCode::NO_CONTENT,
        [(
            header::SET_COOKIE,
            session_cookie::clear_cookie(state.secure_cookie),
        )],
    )
        .into_response())
}

pub(crate) fn disabled() -> AppError {
    AppError::not_found("authentication", "local")
}
