use std::sync::Arc;
use axum::{extract::{Path, State}, http::{HeaderMap, StatusCode}, Json};
use serde_json::{json, Value};

use crate::AppState;
use crate::application::dto::{LoginRequest, LoginResponse, MfaVerifyRequest, SubjectResponse};
use crate::domain::model::SubjectType;
use crate::infrastructure::auth::{self, Claims};
use super::subject_handler::map_domain_error;

pub async fn login(
    Path(subject_type_str): Path<String>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let subject_type = parse_subject_type(&subject_type_str)?;

    let device_name = headers.get("X-Device-Name")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let device_ip = headers.get("X-Forwarded-For")
        .or_else(|| headers.get("X-Real-IP"))
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let user_agent = headers.get("User-Agent")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    let result = state
        .auth_service
        .login(&req.username, &req.password, subject_type, device_name, device_ip, user_agent)
        .await
        .map_err(map_domain_error)?;

    let resp = LoginResponse {
        token: result.token,
        subject_id: result.subject_id,
        subject_type: result.subject_type,
        requires_mfa: result.requires_mfa,
        mfa_token: result.mfa_token,
    };

    Ok(Json(json!({ "success": true, "data": resp })))
}

pub async fn mfa_verify(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<MfaVerifyRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let device_name = headers.get("X-Device-Name").and_then(|v| v.to_str().ok()).map(String::from);
    let device_ip = headers.get("X-Forwarded-For").and_then(|v| v.to_str().ok()).map(String::from);
    let user_agent = headers.get("User-Agent").and_then(|v| v.to_str().ok()).map(String::from);

    let result = state
        .auth_service
        .verify_mfa(&req.mfa_token, &req.code, device_name, device_ip, user_agent)
        .await
        .map_err(map_domain_error)?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "token": result.token,
            "subject_id": result.subject_id,
            "subject_type": result.subject_type,
        }
    })))
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let token = headers.get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "success": false, "error": "Missing authorization header" })),
        ))?;

    let claims = auth::verify_token(token, &state.jwt_secret).map_err(|_| (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "success": false, "error": "Invalid or expired token" })),
    ))?;

    state.auth_service.logout(claims.session_id, token).await.map_err(map_domain_error)?;

    Ok(Json(json!({ "success": true, "data": null })))
}

pub async fn me(
    claims: Claims,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let subject = state
        .auth_service
        .get_subject(claims.sub)
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

    Ok(Json(json!({ "success": true, "data": resp })))
}

fn parse_subject_type(s: &str) -> Result<SubjectType, (StatusCode, Json<Value>)> {
    match s {
        "member" => Ok(SubjectType::Member),
        "staff" => Ok(SubjectType::CommunityStaff),
        "admin" => Ok(SubjectType::PlatformStaff),
        _ => Err((StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "Invalid subject type" })))),
    }
}
