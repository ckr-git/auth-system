use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::error::DomainError;
use crate::domain::repository::CredentialRepository;

pub struct PgCredentialRepository {
    pool: PgPool,
}

impl PgCredentialRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CredentialRepository for PgCredentialRepository {
    async fn create_password(&self, subject_id: Uuid, password_hash: &str) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO credentials (subject_id, credential_type, credential_data)
            VALUES ($1, 'password', $2)
            ON CONFLICT (subject_id, credential_type)
            WHERE credential_type IN ('password', 'totp')
            DO UPDATE SET credential_data = $2, updated_at = NOW()
            "#,
        )
        .bind(subject_id)
        .bind(password_hash)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_password_hash(&self, subject_id: Uuid) -> Result<Option<String>, DomainError> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT credential_data FROM credentials WHERE subject_id = $1 AND credential_type = 'password' AND is_active = TRUE",
        )
        .bind(subject_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    async fn create_totp(&self, subject_id: Uuid, secret_data: &str) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            INSERT INTO credentials (subject_id, credential_type, credential_data)
            VALUES ($1, 'totp', $2)
            ON CONFLICT (subject_id, credential_type)
            WHERE credential_type IN ('password', 'totp')
            DO UPDATE SET credential_data = $2, updated_at = NOW()
            "#,
        )
        .bind(subject_id)
        .bind(secret_data)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_totp_secret(&self, subject_id: Uuid) -> Result<Option<String>, DomainError> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT credential_data FROM credentials WHERE subject_id = $1 AND credential_type = 'totp' AND is_active = TRUE",
        )
        .bind(subject_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    async fn has_credential(&self, subject_id: Uuid, credential_type: &str) -> Result<bool, DomainError> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT COUNT(*) FROM credentials WHERE subject_id = $1 AND credential_type = $2::credential_type AND is_active = TRUE",
        )
        .bind(subject_id)
        .bind(credential_type)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0 > 0).unwrap_or(false))
    }

    async fn create_passkey(&self, subject_id: Uuid, passkey_data: &str) -> Result<Uuid, DomainError> {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO credentials (id, subject_id, credential_type, credential_data) VALUES ($1, $2, 'passkey', $3)",
        )
        .bind(id)
        .bind(subject_id)
        .bind(passkey_data)
        .execute(&self.pool)
        .await?;
        Ok(id)
    }

    async fn find_passkeys(&self, subject_id: Uuid) -> Result<Vec<(Uuid, String)>, DomainError> {
        let rows: Vec<(Uuid, String)> = sqlx::query_as(
            "SELECT id, credential_data FROM credentials WHERE subject_id = $1 AND credential_type = 'passkey' AND is_active = TRUE",
        )
        .bind(subject_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    async fn update_passkey(&self, credential_id: Uuid, passkey_data: &str) -> Result<(), DomainError> {
        sqlx::query(
            "UPDATE credentials SET credential_data = $1, updated_at = NOW() WHERE id = $2 AND credential_type = 'passkey'",
        )
        .bind(passkey_data)
        .bind(credential_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

}
