use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::error::DomainError;
use crate::domain::model::{Subject, SubjectType};
use crate::domain::repository::SubjectRepository;

pub struct PgSubjectRepository {
    pool: PgPool,
}

impl PgSubjectRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn map_subject(row: sqlx::postgres::PgRow) -> Result<Subject, sqlx::Error> {
    use sqlx::Row;
    Ok(Subject {
        id: row.try_get("id")?,
        username: row.try_get("username")?,
        display_name: row.try_get("display_name")?,
        subject_type: row.try_get("subject_type")?,
        is_active: row.try_get("is_active")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[async_trait]
impl SubjectRepository for PgSubjectRepository {
    async fn create(&self, subject: &Subject) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO subjects (id, username, display_name, subject_type, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(subject.id)
        .bind(&subject.username)
        .bind(&subject.display_name)
        .bind(subject.subject_type)
        .bind(subject.is_active)
        .bind(subject.created_at)
        .bind(subject.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Subject>, DomainError> {
        let result = sqlx::query("SELECT * FROM subjects WHERE id = $1")
            .bind(id)
            .try_map(map_subject)
            .fetch_optional(&self.pool)
            .await?;
        Ok(result)
    }

    async fn find_by_username_and_type(
        &self,
        username: &str,
        subject_type: SubjectType,
    ) -> Result<Option<Subject>, DomainError> {
        let result = sqlx::query("SELECT * FROM subjects WHERE username = $1 AND subject_type = $2")
            .bind(username)
            .bind(subject_type)
            .try_map(map_subject)
            .fetch_optional(&self.pool)
            .await?;
        Ok(result)
    }
}
