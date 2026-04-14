use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::domain::model::SubjectType;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub subject_id: Uuid,
    pub subject_type: SubjectType,
    pub requires_mfa: bool,
    pub mfa_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MfaVerifyRequest {
    pub mfa_token: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct PasskeyLoginRequest {
    pub username: String,
    pub subject_type: SubjectType,
}

#[derive(Debug, Deserialize)]
pub struct PasskeyLoginCompleteRequest {
    pub challenge_id: String,
    pub credential: Value,
}
