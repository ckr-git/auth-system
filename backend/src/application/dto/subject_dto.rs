use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::domain::model::SubjectType;

#[derive(Debug, Deserialize)]
pub struct RegisterSubjectRequest {
    pub username: String,
    pub display_name: String,
    pub subject_type: SubjectType,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct SubjectResponse {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub subject_type: SubjectType,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}
