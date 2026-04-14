use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "subject_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SubjectType {
    Member,
    CommunityStaff,
    PlatformStaff,
}

impl std::fmt::Display for SubjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubjectType::Member => write!(f, "member"),
            SubjectType::CommunityStaff => write!(f, "community_staff"),
            SubjectType::PlatformStaff => write!(f, "platform_staff"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub subject_type: SubjectType,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Subject {
    pub fn new(username: String, display_name: String, subject_type: SubjectType) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            username,
            display_name,
            subject_type,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
}
