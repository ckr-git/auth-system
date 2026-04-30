use std::sync::Arc;
use axum::{extract::State, http::{HeaderMap, StatusCode}, Json};
use serde_json::{json, Value};
use webauthn_rs::prelude::*;

use crate::AppState;
use crate::application::dto::{PasskeyLoginRequest, PasskeyLoginCompleteRequest, LoginResponse};
use crate::infrastructure::auth::Claims;
use crate::presentation::rejection::AppJson;
use super::subject_handler::map_domain_error;

pub async fn register_begin(
    claims: Claims,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let subject = state.auth_service.get_subject(claims.sub).await.map_err(map_domain_error)?;

    let ccr = state
        .passkey_service
        .start_registration(claims.sub, &subject.username, &subject.display_name)
        .await
        .map_err(map_domain_error)?;

    Ok(Json(json!({ "success": true, "data": ccr })))
}

pub async fn register_complete(
    claims: Claims,
    State(state): State<Arc<AppState>>,
    AppJson(reg): AppJson<RegisterPublicKeyCredential>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    state
        .passkey_service
        .finish_registration(claims.sub, reg)
        .await
        .map_err(map_domain_error)?;

    Ok(Json(json!({ "success": true, "data": null })))
}

pub async fn authenticate_begin(
    claims: Claims,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let rcr = state
        .passkey_service
        .start_authentication(claims.sub)
        .await
        .map_err(map_domain_error)?;

    Ok(Json(json!({ "success": true, "data": rcr })))
}

pub async fn authenticate_complete(
    claims: Claims,
    State(state): State<Arc<AppState>>,
    AppJson(auth): AppJson<PublicKeyCredential>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    state
        .passkey_service
        .finish_authentication(claims.sub, auth)
        .await
        .map_err(map_domain_error)?;

    Ok(Json(json!({ "success": true, "data": null })))
}

pub async fn login_begin(
    State(state): State<Arc<AppState>>,
    AppJson(req): AppJson<PasskeyLoginRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (challenge_id, rcr) = state
        .passkey_service
        .start_login(&req.username, req.subject_type)
        .await
        .map_err(map_domain_error)?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "challenge_id": challenge_id,
            "options": rcr,
        }
    })))
}

pub async fn login_complete(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AppJson(req): AppJson<PasskeyLoginCompleteRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let credential: PublicKeyCredential = serde_json::from_value(req.credential)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": format!("Invalid credential: {}", e) }))))?;

    let subject_id = state
        .passkey_service
        .finish_login(&req.challenge_id, credential)
        .await
        .map_err(map_domain_error)?;

    let device_name = headers.get("X-Device-Name").and_then(|v| v.to_str().ok()).map(String::from);
    let device_ip = headers.get("X-Forwarded-For").and_then(|v| v.to_str().ok()).map(String::from);
    let user_agent = headers.get("User-Agent").and_then(|v| v.to_str().ok()).map(String::from);

    let result = state
        .auth_service
        .issue_session_for_subject(subject_id, device_name, device_ip, user_agent)
        .await
        .map_err(map_domain_error)?;

    let resp = LoginResponse {
        token: result.token,
        subject_id: result.subject_id,
        subject_type: result.subject_type,
        requires_mfa: false,
        mfa_token: None,
    };

    Ok(Json(json!({ "success": true, "data": resp })))
}
