use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::domain::error::DomainError;
use crate::domain::model::{Subject, SubjectType};
use crate::domain::repository::{CredentialRepository, Session, SessionRepository, SubjectRepository};
use crate::infrastructure::auth;

pub struct AuthResult {
    pub token: String,
    pub subject_id: Uuid,
    pub subject_type: SubjectType,
    pub requires_mfa: bool,
    pub mfa_token: Option<String>,
}

pub struct AuthService<S, C, Ss>
where
    S: SubjectRepository,
    C: CredentialRepository,
    Ss: SessionRepository,
{
    subject_repo: S,
    credential_repo: C,
    session_repo: Ss,
    redis: redis::aio::ConnectionManager,
    jwt_secret: String,
}

impl<S, C, Ss> AuthService<S, C, Ss>
where
    S: SubjectRepository,
    C: CredentialRepository,
    Ss: SessionRepository,
{
    pub fn new(
        subject_repo: S,
        credential_repo: C,
        session_repo: Ss,
        redis: redis::aio::ConnectionManager,
        jwt_secret: String,
    ) -> Self {
        Self { subject_repo, credential_repo, session_repo, redis, jwt_secret }
    }

    pub fn redis(&self) -> &redis::aio::ConnectionManager {
        &self.redis
    }

    pub async fn register(
        &self,
        username: String,
        display_name: String,
        subject_type: SubjectType,
        password: String,
    ) -> Result<Subject, DomainError> {
        if username.is_empty() || password.is_empty() {
            return Err(DomainError::InvalidInput("Username and password are required".into()));
        }
        if username.len() < 3 || username.len() > 32 {
            return Err(DomainError::InvalidInput("Username must be 3-32 characters".into()));
        }
        if !username.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            return Err(DomainError::InvalidInput("Username may only contain letters, digits, underscores, and hyphens".into()));
        }
        if password.len() < 6 {
            return Err(DomainError::InvalidInput("Password must be at least 6 characters".into()));
        }

        let existing = self.subject_repo.find_by_username_and_type(&username, subject_type).await?;
        if existing.is_some() {
            return Err(DomainError::AlreadyExists(
                "Username already exists for this subject type".into(),
            ));
        }

        let subject = Subject::new(username, display_name, subject_type);
        let password_hash = auth::hash_password(&password)
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        self.subject_repo.create(&subject).await?;
        self.credential_repo.create_password(subject.id, &password_hash).await?;

        Ok(subject)
    }

    pub async fn login(
        &self,
        username: &str,
        password: &str,
        subject_type: SubjectType,
        device_name: Option<String>,
        device_ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuthResult, DomainError> {
        let subject = self
            .subject_repo
            .find_by_username_and_type(username, subject_type)
            .await?
            .ok_or_else(|| DomainError::AuthenticationFailed("Invalid credentials".into()))?;

        if !subject.is_active {
            return Err(DomainError::AuthenticationFailed("Account is disabled".into()));
        }

        let password_hash = self
            .credential_repo
            .find_password_hash(subject.id)
            .await?
            .ok_or_else(|| DomainError::AuthenticationFailed("Invalid credentials".into()))?;

        let valid = auth::verify_password(password, &password_hash)
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;
        if !valid {
            return Err(DomainError::AuthenticationFailed("Invalid credentials".into()));
        }

        let has_totp = self.credential_repo.has_credential(subject.id, "totp").await?;
        if has_totp {
            let mfa_token = Uuid::new_v4().to_string();
            let mut redis = self.redis.clone();
            let key = format!("mfa:{}", mfa_token);
            redis::cmd("SETEX")
                .arg(&key)
                .arg(300i64)
                .arg(subject.id.to_string())
                .query_async::<()>(&mut redis)
                .await
                .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

            return Ok(AuthResult {
                token: String::new(),
                subject_id: subject.id,
                subject_type: subject.subject_type,
                requires_mfa: true,
                mfa_token: Some(mfa_token),
            });
        }

        let token = self.issue_session(&subject, device_name, device_ip, user_agent).await?;
        Ok(AuthResult {
            token,
            subject_id: subject.id,
            subject_type: subject.subject_type,
            requires_mfa: false,
            mfa_token: None,
        })
    }

    pub async fn verify_mfa(
        &self,
        mfa_token: &str,
        code: &str,
        device_name: Option<String>,
        device_ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuthResult, DomainError> {
        let mut redis = self.redis.clone();
        let key = format!("mfa:{}", mfa_token);

        let subject_id_str: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut redis)
            .await
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        let subject_id_str = subject_id_str
            .ok_or_else(|| DomainError::AuthenticationFailed("MFA token expired or invalid".into()))?;
        let subject_id: Uuid = subject_id_str
            .parse()
            .map_err(|_| DomainError::Infrastructure("Invalid subject ID in MFA token".into()))?;

        let secret_str = self
            .credential_repo
            .find_totp_secret(subject_id)
            .await?
            .ok_or_else(|| DomainError::Infrastructure("TOTP not configured".into()))?;

        let secret = totp_rs::Secret::Encoded(secret_str)
            .to_bytes()
            .map_err(|_| DomainError::Infrastructure("Invalid TOTP secret".into()))?;
        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1, 6, 1, 30, secret,
            Some("AuthSystem".to_string()), subject_id.to_string(),
        )
        .map_err(|_| DomainError::Infrastructure("Failed to create TOTP".into()))?;

        if !totp.check_current(code).unwrap_or(false) {
            return Err(DomainError::AuthenticationFailed("Invalid TOTP code".into()));
        }

        redis::cmd("DEL")
            .arg(&key)
            .query_async::<()>(&mut redis)
            .await
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        let subject = self
            .subject_repo
            .find_by_id(subject_id)
            .await?
            .ok_or_else(|| DomainError::NotFound("Subject not found".into()))?;

        let token = self.issue_session(&subject, device_name, device_ip, user_agent).await?;
        Ok(AuthResult {
            token,
            subject_id: subject.id,
            subject_type: subject.subject_type,
            requires_mfa: false,
            mfa_token: None,
        })
    }

    pub async fn get_subject(&self, subject_id: Uuid) -> Result<Subject, DomainError> {
        self.subject_repo
            .find_by_id(subject_id)
            .await?
            .ok_or_else(|| DomainError::NotFound("Subject not found".into()))
    }

    pub async fn logout(&self, session_id: Uuid, token: &str) -> Result<(), DomainError> {
        let token_hash = hash_token(token);
        self.session_repo.deactivate_by_token_hash(&token_hash).await?;

        let mut redis = self.redis.clone();
        let key = format!("session:{}", session_id);
        redis::cmd("DEL")
            .arg(&key)
            .query_async::<()>(&mut redis)
            .await
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(())
    }

    pub async fn issue_session_for_subject(
        &self,
        subject_id: Uuid,
        device_name: Option<String>,
        device_ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<AuthResult, DomainError> {
        let subject = self.subject_repo
            .find_by_id(subject_id)
            .await?
            .ok_or_else(|| DomainError::NotFound("Subject not found".into()))?;

        let token = self.issue_session(&subject, device_name, device_ip, user_agent).await?;
        Ok(AuthResult {
            token,
            subject_id: subject.id,
            subject_type: subject.subject_type,
            requires_mfa: false,
            mfa_token: None,
        })
    }

    async fn issue_session(
        &self,
        subject: &Subject,
        device_name: Option<String>,
        device_ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<String, DomainError> {
        let session_id = Uuid::new_v4();
        let token = auth::create_token(subject.id, subject.subject_type, session_id, &self.jwt_secret)
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        let now = Utc::now();
        let token_hash = hash_token(&token);

        let session = Session {
            id: session_id,
            subject_id: subject.id,
            device_name,
            device_ip,
            user_agent,
            token_hash,
            is_active: true,
            created_at: now,
            expires_at: now + Duration::hours(24),
            last_active_at: now,
        };
        self.session_repo.create(&session).await?;

        let mut redis = self.redis.clone();
        let key = format!("session:{}", session_id);
        redis::cmd("SETEX")
            .arg(&key)
            .arg(86400i64)
            .arg(subject.id.to_string())
            .query_async::<()>(&mut redis)
            .await
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(token)
    }
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}
