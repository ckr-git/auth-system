use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::error::DomainError;
use crate::domain::repository::session_repo::{Session, SessionRepository};

pub struct PgSessionRepository {
    pool: PgPool,
}

impl PgSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn map_session(row: sqlx::postgres::PgRow) -> Result<Session, sqlx::Error> {
    use sqlx::Row;
    Ok(Session {
        id: row.try_get("id")?,
        subject_id: row.try_get("subject_id")?,
        device_name: row.try_get("device_name")?,
        device_ip: row.try_get("device_ip")?,
        user_agent: row.try_get("user_agent")?,
        token_hash: row.try_get("token_hash")?,
        is_active: row.try_get("is_active")?,
        created_at: row.try_get("created_at")?,
        expires_at: row.try_get("expires_at")?,
        last_active_at: row.try_get("last_active_at")?,
    })
}

#[async_trait]
impl SessionRepository for PgSessionRepository {
    async fn create(&self, session: &Session) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO sessions (id, subject_id, device_name, device_ip, user_agent, token_hash, is_active, created_at, expires_at, last_active_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(session.id)
        .bind(session.subject_id)
        .bind(&session.device_name)
        .bind(&session.device_ip)
        .bind(&session.user_agent)
        .bind(&session.token_hash)
        .bind(session.is_active)
        .bind(session.created_at)
        .bind(session.expires_at)
        .bind(session.last_active_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_active_by_subject(&self, subject_id: Uuid) -> Result<Vec<Session>, DomainError> {
        let rows = sqlx::query(
            "SELECT * FROM sessions WHERE subject_id = $1 AND is_active = TRUE AND expires_at > NOW() ORDER BY last_active_at DESC",
        )
        .bind(subject_id)
        .try_map(map_session)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    async fn deactivate(&self, session_id: Uuid, subject_id: Uuid) -> Result<(), DomainError> {
        sqlx::query(
            "UPDATE sessions SET is_active = FALSE WHERE id = $1 AND subject_id = $2",
        )
        .bind(session_id)
        .bind(subject_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn deactivate_by_token_hash(&self, token_hash: &str) -> Result<(), DomainError> {
        sqlx::query("UPDATE sessions SET is_active = FALSE WHERE token_hash = $1")
            .bind(token_hash)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn touch(&self, session_id: Uuid) -> Result<(), DomainError> {
        sqlx::query(
            "UPDATE sessions SET last_active_at = NOW() WHERE id = $1 AND is_active = TRUE AND expires_at > NOW()",
        )
        .bind(session_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
