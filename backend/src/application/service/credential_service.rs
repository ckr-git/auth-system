use uuid::Uuid;

use crate::domain::error::DomainError;
use crate::domain::repository::CredentialRepository;

pub struct CredentialStatus {
    pub has_password: bool,
    pub has_totp: bool,
    pub passkey_count: usize,
}

pub struct TotpSetupResult {
    pub qr_code_base64: String,
    pub secret: String,
}

pub struct CredentialService<C: CredentialRepository> {
    credential_repo: C,
    redis: redis::aio::ConnectionManager,
}

impl<C: CredentialRepository> CredentialService<C> {
    pub fn new(credential_repo: C, redis: redis::aio::ConnectionManager) -> Self {
        Self { credential_repo, redis }
    }

    pub async fn setup_totp(&self, subject_id: Uuid) -> Result<TotpSetupResult, DomainError> {
        let secret = totp_rs::Secret::generate_secret();
        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1, 6, 1, 30,
            secret.to_bytes().map_err(|_| DomainError::Infrastructure("Failed to generate TOTP secret".into()))?,
            Some("AuthSystem".to_string()), subject_id.to_string(),
        )
        .map_err(|_| DomainError::Infrastructure("Failed to create TOTP".into()))?;

        let qr_code = totp.get_qr_base64()
            .map_err(|_| DomainError::Infrastructure("Failed to generate QR code".into()))?;
        let secret_base32 = secret.to_encoded().to_string();

        // Store pending secret in Redis (5 min TTL), NOT in DB yet
        let mut redis = self.redis.clone();
        let key = format!("totp_pending:{}", subject_id);
        redis::cmd("SETEX")
            .arg(&key)
            .arg(300i64)
            .arg(&secret_base32)
            .query_async::<()>(&mut redis)
            .await
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(TotpSetupResult { qr_code_base64: qr_code, secret: secret_base32 })
    }

    pub async fn confirm_totp(&self, subject_id: Uuid, code: &str) -> Result<bool, DomainError> {
        let mut redis = self.redis.clone();
        let key = format!("totp_pending:{}", subject_id);

        let secret_str: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut redis)
            .await
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        let secret_str = secret_str
            .ok_or_else(|| DomainError::InvalidInput("No pending TOTP setup or it has expired. Please start setup again.".into()))?;

        let secret = totp_rs::Secret::Encoded(secret_str.clone())
            .to_bytes()
            .map_err(|_| DomainError::Infrastructure("Invalid TOTP secret".into()))?;
        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1, 6, 1, 30, secret,
            Some("AuthSystem".to_string()), subject_id.to_string(),
        )
        .map_err(|_| DomainError::Infrastructure("Failed to create TOTP".into()))?;

        if !totp.check_current(code).unwrap_or(false) {
            return Ok(false);
        }

        // Code is valid — persist to DB and clean up Redis
        self.credential_repo.create_totp(subject_id, &secret_str).await?;
        let _: () = redis::cmd("DEL").arg(&key).query_async(&mut redis).await.unwrap_or(());

        Ok(true)
    }

    pub async fn verify_totp(&self, subject_id: Uuid, code: &str) -> Result<bool, DomainError> {
        let secret_str = self.credential_repo.find_totp_secret(subject_id).await?
            .ok_or_else(|| DomainError::NotFound("TOTP not configured".into()))?;

        let secret = totp_rs::Secret::Encoded(secret_str)
            .to_bytes()
            .map_err(|_| DomainError::Infrastructure("Invalid TOTP secret".into()))?;
        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1, 6, 1, 30, secret,
            Some("AuthSystem".to_string()), subject_id.to_string(),
        )
        .map_err(|_| DomainError::Infrastructure("Failed to create TOTP".into()))?;

        Ok(totp.check_current(code).unwrap_or(false))
    }

    pub async fn get_status(&self, subject_id: Uuid) -> Result<CredentialStatus, DomainError> {
        let has_password = self.credential_repo.has_credential(subject_id, "password").await?;
        let has_totp = self.credential_repo.has_credential(subject_id, "totp").await?;
        let passkeys = self.credential_repo.find_passkeys(subject_id).await?;

        Ok(CredentialStatus { has_password, has_totp, passkey_count: passkeys.len() })
    }
}
