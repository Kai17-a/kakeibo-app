use crate::{
    auth::{AuthState, middleware::CurrentUser, now},
    handler::auth::{disabled, signed_in},
    model::auth::{
        PasskeyChallenge, PasskeyLoginRequest, PasskeyRegistrationRequest, PasskeySummary,
    },
    service::passkeys::PasskeyService,
    utils::error::{AppError, AppResult},
};
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use serde::Serialize;

fn passkeys(state: &AuthState) -> AppResult<&PasskeyService> {
    state.passkeys.as_ref().ok_or_else(disabled)
}

/// The middleware adds the user for `/api/passkeys`; it is absent only when authentication is
/// off, which `passkeys` already reports as not found.
fn signed_in_user(user: Option<Extension<CurrentUser>>) -> AppResult<CurrentUser> {
    user.map(|Extension(user)| user)
        .ok_or_else(|| AppError::unauthorized("authentication required"))
}

fn challenge(challenge_id: String, options: impl Serialize) -> AppResult<Json<PasskeyChallenge>> {
    let options = serde_json::to_value(options)
        .map_err(|error| AppError::context("Failed to encode passkey options", error))?;
    Ok(Json(PasskeyChallenge {
        challenge_id,
        options,
    }))
}

#[utoipa::path(get,path="/api/passkeys",responses((status=200,body=Vec<PasskeySummary>),(status=401),(status=404)))]
pub async fn list(
    State(state): State<AuthState>,
    user: Option<Extension<CurrentUser>>,
) -> AppResult<Json<Vec<PasskeySummary>>> {
    let service = passkeys(&state)?;
    let user = signed_in_user(user)?;
    Ok(Json(service.list(&user.user_id).await?))
}

#[utoipa::path(post,path="/api/passkeys/registration",responses((status=200,body=PasskeyChallenge),(status=401),(status=404)))]
pub async fn start_registration(
    State(state): State<AuthState>,
    user: Option<Extension<CurrentUser>>,
) -> AppResult<Json<PasskeyChallenge>> {
    let service = passkeys(&state)?;
    let user = signed_in_user(user)?;
    let (challenge_id, options) = service
        .start_registration(&user.user_id, &user.username, now())
        .await?;
    challenge(challenge_id, options)
}

#[utoipa::path(post,path="/api/passkeys",request_body=PasskeyRegistrationRequest,responses((status=201,body=PasskeySummary),(status=400),(status=401),(status=404),(status=409)))]
pub async fn register(
    State(state): State<AuthState>,
    user: Option<Extension<CurrentUser>>,
    Json(request): Json<PasskeyRegistrationRequest>,
) -> AppResult<(StatusCode, Json<PasskeySummary>)> {
    let service = passkeys(&state)?;
    let user = signed_in_user(user)?;
    let summary = service
        .finish_registration(
            &user.user_id,
            &request.challenge_id,
            &request.name,
            &request.credential,
            now(),
        )
        .await?;
    Ok((StatusCode::CREATED, Json(summary)))
}

#[utoipa::path(delete,path="/api/passkeys/{id}",params(("id"=String,Path)),responses((status=204),(status=401),(status=404)))]
pub async fn delete(
    State(state): State<AuthState>,
    user: Option<Extension<CurrentUser>>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    let service = passkeys(&state)?;
    let user = signed_in_user(user)?;
    service.delete(&user.user_id, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(post,path="/api/auth/passkey/challenge",responses((status=200,body=PasskeyChallenge),(status=404)))]
pub async fn start_login(State(state): State<AuthState>) -> AppResult<Json<PasskeyChallenge>> {
    let (challenge_id, options) = passkeys(&state)?.start_login(now())?;
    challenge(challenge_id, options)
}

#[utoipa::path(post,path="/api/auth/passkey/login",request_body=PasskeyLoginRequest,responses((status=204),(status=400),(status=401),(status=404)))]
pub async fn login(
    State(state): State<AuthState>,
    headers: HeaderMap,
    Json(request): Json<PasskeyLoginRequest>,
) -> AppResult<Response> {
    let service = state.service.as_ref().ok_or_else(disabled)?;
    let current = now();
    let user_id = passkeys(&state)?
        .finish_login(&request.challenge_id, &request.credential, current)
        .await?;
    let token = service.issue_session(&user_id, current).await?;
    signed_in(&state, service, &headers, &token).await
}
