use std::sync::Arc;
use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};

use crate::AppState;
use crate::application::dto::{TotpVerifyRequest, TotpSetupResponse, CredentialStatusResponse};
use crate::infrastructure::auth::Claims;
use super::subject_handler::map_domain_error;

pub async fn totp_setup(
    claims: Claims,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let result = state.credential_service.setup_totp(claims.sub).await.map_err(map_domain_error)?;

    let resp = TotpSetupResponse {
        qr_code_base64: result.qr_code_base64,
        secret: result.secret,
    };

    Ok(Json(json!({ "success": true, "data": resp })))
}

pub async fn totp_confirm(
    claims: Claims,
    State(state): State<Arc<AppState>>,
    Json(req): Json<TotpVerifyRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let valid = state.credential_service.confirm_totp(claims.sub, &req.code).await.map_err(map_domain_error)?;

    if !valid {
        return Err((StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": "Invalid TOTP code. Please try again." }))));
    }

    Ok(Json(json!({ "success": true, "data": { "confirmed": true } })))
}

pub async fn totp_verify(
    claims: Claims,
    State(state): State<Arc<AppState>>,
    Json(req): Json<TotpVerifyRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let valid = state.credential_service.verify_totp(claims.sub, &req.code).await.map_err(map_domain_error)?;

    Ok(Json(json!({ "success": true, "data": { "valid": valid } })))
}

pub async fn credential_status(
    claims: Claims,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let status = state.credential_service.get_status(claims.sub).await.map_err(map_domain_error)?;

    let resp = CredentialStatusResponse {
        has_password: status.has_password,
        has_totp: status.has_totp,
        passkey_count: status.passkey_count,
    };

    Ok(Json(json!({ "success": true, "data": resp })))
}
