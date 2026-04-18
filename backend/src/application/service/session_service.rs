use uuid::Uuid;

use crate::domain::error::DomainError;
use crate::domain::repository::{Session, SessionRepository};

pub struct SessionService<Ss: SessionRepository> {
    session_repo: Ss,
    redis: redis::aio::ConnectionManager,
}

impl<Ss: SessionRepository> SessionService<Ss> {
    pub fn new(session_repo: Ss, redis: redis::aio::ConnectionManager) -> Self {
        Self { session_repo, redis }
    }

    pub async fn list_active(&self, subject_id: Uuid) -> Result<Vec<Session>, DomainError> {
        self.session_repo.find_active_by_subject(subject_id).await
    }

    pub async fn touch(&self, session_id: Uuid) -> Result<(), DomainError> {
        self.session_repo.touch(session_id).await
    }

    pub async fn revoke(&self, session_id: Uuid, subject_id: Uuid) -> Result<(), DomainError> {
        self.session_repo.deactivate(session_id, subject_id).await?;

        let mut redis = self.redis.clone();
        let key = format!("session:{}", session_id);
        redis::cmd("DEL")
            .arg(&key)
            .query_async::<()>(&mut redis)
            .await
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(())
    }
}
