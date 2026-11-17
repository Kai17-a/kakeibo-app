use crate::{
    auth::challenge_store::ChallengeStore,
    database::models::auth::PasskeyRow,
    model::auth::PasskeySummary,
    repository::auth::AuthRepository,
    utils::error::{AppError, AppResult},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use std::sync::Arc;
use tracing::{info, warn};
use webauthn_rs::prelude::{
    CreationChallengeResponse, DiscoverableAuthentication, Passkey, PasskeyRegistration,
    PublicKeyCredential, RegisterPublicKeyCredential, RequestChallengeResponse, Url, Uuid,
    Webauthn, WebauthnBuilder,
};
use webauthn_rs_proto::ResidentKeyRequirement;

pub const MAX_PASSKEY_NAME_LENGTH: usize = 64;
const RP_NAME: &str = "Kakeibo";
const LOGIN_FAILED: &str = "passkey login failed";

/// Registration state remembers whose ceremony it is.
struct PendingRegistration {
    user_id: String,
    state: PasskeyRegistration,
}

#[derive(Clone)]
pub struct PasskeyService {
    repository: AuthRepository,
    webauthn: Arc<Webauthn>,
    registrations: Arc<ChallengeStore<PendingRegistration>>,
    authentications: Arc<ChallengeStore<DiscoverableAuthentication>>,
}

impl PasskeyService {
    /// Builds the service for the site at `public_origin` (`scheme://host[:port]`). Fails when
    /// the host cannot be a WebAuthn relying party ID, such as an IP address.
    pub fn new(repository: AuthRepository, public_origin: &str) -> AppResult<Self> {
        let origin = Url::parse(public_origin)
            .map_err(|error| AppError::context("Invalid public URL for passkeys", error))?;
        let rp_id = origin
            .host_str()
            .ok_or_else(|| AppError::bad_request("public URL has no host"))?
            .to_owned();
        let webauthn = WebauthnBuilder::new(&rp_id, &origin)
            .and_then(|builder| builder.rp_name(RP_NAME).build())
            .map_err(|error| AppError::context("Failed to configure passkeys", error))?;
        Ok(Self {
            repository,
            webauthn: Arc::new(webauthn),
            registrations: Arc::default(),
            authentications: Arc::default(),
        })
    }

    pub async fn list(&self, user_id: &str) -> AppResult<Vec<PasskeySummary>> {
        Ok(self
            .repository
            .find_passkeys(user_id)
            .await?
            .into_iter()
            .map(summary)
            .collect())
    }

    pub async fn start_registration(
        &self,
        user_id: &str,
        username: &str,
        now: i64,
    ) -> AppResult<(String, CreationChallengeResponse)> {
        let user_handle = user_handle(user_id)?;
        let existing = self
            .repository
            .find_passkeys(user_id)
            .await?
            .iter()
            .map(|row| deserialize(row).map(|passkey| passkey.cred_id().clone()))
            .collect::<AppResult<Vec<_>>>()?;
        let (mut options, state) = self
            .webauthn
            .start_passkey_registration(user_handle, username, username, Some(existing))
            .map_err(|error| AppError::context("Failed to start passkey registration", error))?;
        // Login does not ask for a username, so the authenticator must store the credential
        // (a discoverable credential) to find it later.
        if let Some(selection) = options.public_key.authenticator_selection.as_mut() {
            selection.resident_key = Some(ResidentKeyRequirement::Required);
            selection.require_resident_key = true;
        }
        let challenge_id = self.registrations.insert(
            PendingRegistration {
                user_id: user_id.to_owned(),
                state,
            },
            now,
        )?;
        Ok((challenge_id, options))
    }

    pub async fn finish_registration(
        &self,
        user_id: &str,
        challenge_id: &str,
        name: &str,
        credential: &RegisterPublicKeyCredential,
        now: i64,
    ) -> AppResult<PasskeySummary> {
        let name = validate_name(name)?;
        let pending = self
            .registrations
            .take(challenge_id, now)
            .filter(|pending| pending.user_id == user_id)
            .ok_or_else(expired_challenge)?;
        let passkey = self
            .webauthn
            .finish_passkey_registration(credential, &pending.state)
            .map_err(|error| {
                warn!(%error, "Passkey registration rejected");
                AppError::bad_request("passkey registration failed")
            })?;
        let id = URL_SAFE_NO_PAD.encode(passkey.cred_id());
        let serialized = serde_json::to_string(&passkey)
            .map_err(|error| AppError::context("Failed to store the passkey", error))?;
        self.repository
            .insert_passkey(&id, user_id, name, &serialized, now)
            .await?;
        info!(user_id, "Passkey registered");
        Ok(PasskeySummary {
            id,
            name: name.to_owned(),
            created_at: now,
            last_used_at: None,
        })
    }

    pub async fn delete(&self, user_id: &str, id: &str) -> AppResult<()> {
        if self.repository.delete_passkey(id, user_id).await? {
            Ok(())
        } else {
            Err(AppError::not_found("passkey", id))
        }
    }

    pub fn start_login(&self, now: i64) -> AppResult<(String, RequestChallengeResponse)> {
        let (mut options, state) = self
            .webauthn
            .start_discoverable_authentication()
            .map_err(|error| AppError::context("Failed to start passkey login", error))?;
        // The login screen asks through an explicit button, not through form autofill.
        options.mediation = None;
        let challenge_id = self.authentications.insert(state, now)?;
        Ok((challenge_id, options))
    }

    /// Verifies the assertion and returns the ID of the user it belongs to.
    pub async fn finish_login(
        &self,
        challenge_id: &str,
        credential: &PublicKeyCredential,
        now: i64,
    ) -> AppResult<String> {
        let state = self
            .authentications
            .take(challenge_id, now)
            .ok_or_else(expired_challenge)?;
        let (user_handle, credential_id) = self
            .webauthn
            .identify_discoverable_authentication(credential)
            .map_err(|_| AppError::unauthorized(LOGIN_FAILED))?;
        let user_id = user_handle.simple().to_string();
        let id = URL_SAFE_NO_PAD.encode(credential_id);
        let Some(row) = self.repository.find_passkey(&id, &user_id).await? else {
            info!("Passkey login with an unknown credential");
            return Err(AppError::unauthorized(LOGIN_FAILED));
        };
        let mut passkey = deserialize(&row)?;
        let result = self
            .webauthn
            .finish_discoverable_authentication_v2(credential, state, &passkey)
            .map_err(|error| {
                info!(%error, "Passkey login rejected");
                AppError::unauthorized(LOGIN_FAILED)
            })?;
        passkey.update_credential(&result);
        let serialized = serde_json::to_string(&passkey)
            .map_err(|error| AppError::context("Failed to store the passkey", error))?;
        self.repository
            .update_passkey_usage(&id, &user_id, &serialized, now)
            .await?;
        Ok(user_id)
    }
}

fn summary(row: PasskeyRow) -> PasskeySummary {
    PasskeySummary {
        id: row.id,
        name: row.name,
        created_at: row.created_at,
        last_used_at: row.last_used_at,
    }
}

fn deserialize(row: &PasskeyRow) -> AppResult<Passkey> {
    serde_json::from_str(&row.credential)
        .map_err(|error| AppError::context("Failed to read a stored passkey", error))
}

/// User IDs are 32 hexadecimal characters, which is also the simple form of a UUID.
fn user_handle(user_id: &str) -> AppResult<Uuid> {
    Uuid::parse_str(user_id).map_err(|error| AppError::context("Invalid user ID", error))
}

fn validate_name(name: &str) -> AppResult<&str> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::bad_request("passkey name is required"));
    }
    if name.chars().count() > MAX_PASSKEY_NAME_LENGTH {
        return Err(AppError::bad_request(&format!(
            "passkey name must be at most {MAX_PASSKEY_NAME_LENGTH} characters"
        )));
    }
    Ok(name)
}

fn expired_challenge() -> AppError {
    AppError::bad_request("passkey request expired; start again")
}
