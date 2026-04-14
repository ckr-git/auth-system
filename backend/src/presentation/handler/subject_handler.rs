use std::sync::Arc;
use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};

use crate::AppState;
use crate::application::dto::{RegisterSubjectRequest, SubjectResponse};
use crate::domain::DomainError;

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterSubjectRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let subject = state
        .auth_service
        .register(req.username, req.display_name, req.subject_type, req.password)
        .await
        .map_err(map_domain_error)?;

    let resp = SubjectResponse {
        id: subject.id,
        username: subject.username,
        display_name: subject.display_name,
        subject_type: subject.subject_type,
        is_active: subject.is_active,
        created_at: subject.created_at,
    };

    Ok((StatusCode::CREATED, Json(json!({ "success": true, "data": resp }))))
}

pub fn map_domain_error(e: DomainError) -> (StatusCode, Json<Value>) {
    let (status, msg) = match &e {
        DomainError::NotFound(m) => (StatusCode::NOT_FOUND, m.as_str()),
        DomainError::AlreadyExists(m) => (StatusCode::CONFLICT, m.as_str()),
        DomainError::InvalidInput(m) => (StatusCode::BAD_REQUEST, m.as_str()),
        DomainError::AuthenticationFailed(m) => (StatusCode::UNAUTHORIZED, m.as_str()),
        DomainError::Infrastructure(_) => {
            tracing::error!("Infrastructure error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        }
    };
    (status, Json(json!({ "success": false, "error": msg })))
}
