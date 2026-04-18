use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::error::DomainError;

#[derive(Debug, Clone)]
pub struct Session {
    pub id: Uuid,
    pub subject_id: Uuid,
    pub device_name: Option<String>,
    pub device_ip: Option<String>,
    pub user_agent: Option<String>,
    pub token_hash: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn create(&self, session: &Session) -> Result<(), DomainError>;
    async fn find_active_by_subject(&self, subject_id: Uuid) -> Result<Vec<Session>, DomainError>;
    async fn deactivate(&self, session_id: Uuid, subject_id: Uuid) -> Result<(), DomainError>;
    async fn deactivate_by_token_hash(&self, token_hash: &str) -> Result<(), DomainError>;
    async fn touch(&self, session_id: Uuid) -> Result<(), DomainError>;
}
