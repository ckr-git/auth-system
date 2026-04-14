use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::error::DomainError;
use crate::domain::model::{Subject, SubjectType};

#[async_trait]
pub trait SubjectRepository: Send + Sync {
    async fn create(&self, subject: &Subject) -> Result<(), DomainError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Subject>, DomainError>;
    async fn find_by_username_and_type(
        &self,
        username: &str,
        subject_type: SubjectType,
    ) -> Result<Option<Subject>, DomainError>;
}
