use uuid::Uuid;
use webauthn_rs::prelude::*;

use crate::domain::error::DomainError;
use crate::domain::model::SubjectType;
use crate::domain::repository::{CredentialRepository, SubjectRepository};

pub struct PasskeyService<C: CredentialRepository, S: SubjectRepository> {
    credential_repo: C,
    subject_repo: S,
    webauthn: webauthn_rs::Webauthn,
    redis: redis::aio::ConnectionManager,
}

impl<C: CredentialRepository, S: SubjectRepository> PasskeyService<C, S> {
    pub fn new(
        credential_repo: C,
        subject_repo: S,
        webauthn: webauthn_rs::Webauthn,
        redis: redis::aio::ConnectionManager,
    ) -> Self {
        Self { credential_repo, subject_repo, webauthn, redis }
    }

    // --- Registration (requires auth) ---

    pub async fn start_registration(
        &self,
        subject_id: Uuid,
        username: &str,
        display_name: &str,
    ) -> Result<CreationChallengeResponse, DomainError> {
        let existing = self.credential_repo.find_passkeys(subject_id).await?;
        let exclude_credentials: Vec<CredentialID> = existing
            .iter()
            .filter_map(|(_, data)| serde_json::from_str::<Passkey>(data).ok())
            .map(|pk| pk.cred_id().clone())
            .collect();

        let (ccr, reg_state) = self
            .webauthn
            .start_passkey_registration(subject_id, username, display_name, Some(exclude_credentials))
            .map_err(|e| DomainError::Infrastructure(format!("WebAuthn error: {}", e)))?;

        let state_json = serde_json::to_string(&reg_state)
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;
        let mut redis = self.redis.clone();
        let key = format!("webauthn_reg:{}", subject_id);
        redis::cmd("SETEX").arg(&key).arg(60i64).arg(&state_json)
            .query_async::<()>(&mut redis).await
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(ccr)
    }

    pub async fn finish_registration(
        &self,
        subject_id: Uuid,
        reg: RegisterPublicKeyCredential,
    ) -> Result<(), DomainError> {
        let mut redis = self.redis.clone();
        let key = format!("webauthn_reg:{}", subject_id);
        let state_json: Option<String> = redis::cmd("GET").arg(&key)
            .query_async(&mut redis).await
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        let state_json = state_json
            .ok_or_else(|| DomainError::AuthenticationFailed("Registration challenge expired".into()))?;
        let reg_state: PasskeyRegistration = serde_json::from_str(&state_json)
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        let passkey = self.webauthn
            .finish_passkey_registration(&reg, &reg_state)
            .map_err(|e| DomainError::AuthenticationFailed(format!("WebAuthn verification failed: {}", e)))?;

        let passkey_json = serde_json::to_string(&passkey)
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;
        self.credential_repo.create_passkey(subject_id, &passkey_json).await?;

        let _: () = redis::cmd("DEL").arg(&key).query_async(&mut redis).await.unwrap_or(());
        Ok(())
    }

    // --- Authentication (requires auth — verify passkey for logged-in user) ---

    pub async fn start_authentication(
        &self,
        subject_id: Uuid,
    ) -> Result<RequestChallengeResponse, DomainError> {
        let existing = self.credential_repo.find_passkeys(subject_id).await?;
        if existing.is_empty() {
            return Err(DomainError::NotFound("No passkeys registered".into()));
        }

        let passkeys: Vec<Passkey> = existing.iter()
            .filter_map(|(_, data)| serde_json::from_str(data).ok())
            .collect();

        let (rcr, auth_state) = self.webauthn
            .start_passkey_authentication(&passkeys)
            .map_err(|e| DomainError::Infrastructure(format!("WebAuthn error: {}", e)))?;

        let state_json = serde_json::to_string(&auth_state)
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;
        let mut redis = self.redis.clone();
        let key = format!("webauthn_auth:{}", subject_id);
        redis::cmd("SETEX").arg(&key).arg(60i64).arg(&state_json)
            .query_async::<()>(&mut redis).await
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(rcr)
    }

    pub async fn finish_authentication(
        &self,
        subject_id: Uuid,
        auth: PublicKeyCredential,
    ) -> Result<(), DomainError> {
        let mut redis = self.redis.clone();
        let key = format!("webauthn_auth:{}", subject_id);
        let state_json: Option<String> = redis::cmd("GET").arg(&key)
            .query_async(&mut redis).await
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        let state_json = state_json
            .ok_or_else(|| DomainError::AuthenticationFailed("Authentication challenge expired".into()))?;
        let auth_state: PasskeyAuthentication = serde_json::from_str(&state_json)
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        let auth_result = self.webauthn
            .finish_passkey_authentication(&auth, &auth_state)
            .map_err(|e| DomainError::AuthenticationFailed(format!("WebAuthn verification failed: {}", e)))?;

        // Update passkey counter in DB
        self.update_passkey_credential(subject_id, &auth_result).await?;

        let _: () = redis::cmd("DEL").arg(&key).query_async(&mut redis).await.unwrap_or(());
        Ok(())
    }

    // --- Passkey Login (no auth required) ---

    pub async fn start_login(
        &self,
        username: &str,
        subject_type: SubjectType,
    ) -> Result<(String, RequestChallengeResponse), DomainError> {
        let subject = self.subject_repo
            .find_by_username_and_type(username, subject_type)
            .await?
            .ok_or_else(|| DomainError::AuthenticationFailed("Invalid credentials".into()))?;

        if !subject.is_active {
            return Err(DomainError::AuthenticationFailed("Account is disabled".into()));
        }

        let existing = self.credential_repo.find_passkeys(subject.id).await?;
        if existing.is_empty() {
            return Err(DomainError::NotFound("No passkeys registered for this user".into()));
        }

        let passkeys: Vec<Passkey> = existing.iter()
            .filter_map(|(_, data)| serde_json::from_str(data).ok())
            .collect();

        let (rcr, auth_state) = self.webauthn
            .start_passkey_authentication(&passkeys)
            .map_err(|e| DomainError::Infrastructure(format!("WebAuthn error: {}", e)))?;

        let challenge_id = Uuid::new_v4().to_string();
        let state_json = serde_json::to_string(&auth_state)
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        let mut redis = self.redis.clone();
        let state_key = format!("webauthn_login:{}", challenge_id);
        redis::cmd("SETEX").arg(&state_key).arg(120i64).arg(&state_json)
            .query_async::<()>(&mut redis).await
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;
        let subject_key = format!("webauthn_login_sub:{}", challenge_id);
        redis::cmd("SETEX").arg(&subject_key).arg(120i64).arg(subject.id.to_string())
            .query_async::<()>(&mut redis).await
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok((challenge_id, rcr))
    }

    pub async fn finish_login(
        &self,
        challenge_id: &str,
        auth: PublicKeyCredential,
    ) -> Result<Uuid, DomainError> {
        let mut redis = self.redis.clone();

        let state_key = format!("webauthn_login:{}", challenge_id);
        let state_json: Option<String> = redis::cmd("GET").arg(&state_key)
            .query_async(&mut redis).await
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;
        let state_json = state_json
            .ok_or_else(|| DomainError::AuthenticationFailed("Login challenge expired".into()))?;

        let subject_key = format!("webauthn_login_sub:{}", challenge_id);
        let subject_id_str: Option<String> = redis::cmd("GET").arg(&subject_key)
            .query_async(&mut redis).await
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;
        let subject_id: Uuid = subject_id_str
            .ok_or_else(|| DomainError::AuthenticationFailed("Login challenge expired".into()))?
            .parse()
            .map_err(|_| DomainError::Infrastructure("Invalid subject ID".into()))?;

        let auth_state: PasskeyAuthentication = serde_json::from_str(&state_json)
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        let auth_result = self.webauthn
            .finish_passkey_authentication(&auth, &auth_state)
            .map_err(|e| DomainError::AuthenticationFailed(format!("WebAuthn verification failed: {}", e)))?;

        // Update passkey counter in DB
        self.update_passkey_credential(subject_id, &auth_result).await?;

        let _: () = redis::cmd("DEL").arg(&state_key).query_async(&mut redis).await.unwrap_or(());
        let _: () = redis::cmd("DEL").arg(&subject_key).query_async(&mut redis).await.unwrap_or(());

        Ok(subject_id)
    }

    async fn update_passkey_credential(
        &self,
        subject_id: Uuid,
        auth_result: &AuthenticationResult,
    ) -> Result<(), DomainError> {
        let existing = self.credential_repo.find_passkeys(subject_id).await?;
        for (cred_id, data) in &existing {
            if let Ok(mut passkey) = serde_json::from_str::<Passkey>(data) {
                if passkey.cred_id() == auth_result.cred_id() {
                    passkey.update_credential(auth_result);
                    let updated_json = serde_json::to_string(&passkey)
                        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;
                    self.credential_repo.update_passkey(*cred_id, &updated_json).await?;
                    break;
                }
            }
        }
        Ok(())
    }
}