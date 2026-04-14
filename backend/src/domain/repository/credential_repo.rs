use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::error::DomainError;

#[async_trait]
pub trait CredentialRepository: Send + Sync {
    async fn create_password(&self, subject_id: Uuid, password_hash: &str) -> Result<(), DomainError>;
    async fn find_password_hash(&self, subject_id: Uuid) -> Result<Option<String>, DomainError>;
    async fn create_totp(&self, subject_id: Uuid, secret_data: &str) -> Result<(), DomainError>;
    async fn find_totp_secret(&self, subject_id: Uuid) -> Result<Option<String>, DomainError>;
    async fn has_credential(&self, subject_id: Uuid, credential_type: &str) -> Result<bool, DomainError>;
    async fn create_passkey(&self, subject_id: Uuid, passkey_data: &str) -> Result<Uuid, DomainError>;
    async fn find_passkeys(&self, subject_id: Uuid) -> Result<Vec<(Uuid, String)>, DomainError>;
    async fn update_passkey(&self, credential_id: Uuid, passkey_data: &str) -> Result<(), DomainError>;
}
