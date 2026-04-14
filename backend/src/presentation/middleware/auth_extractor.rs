use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    Json,
};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::infrastructure::auth;
use crate::AppState;

impl FromRequestParts<Arc<AppState>> for auth::Claims {
    type Rejection = (StatusCode, Json<Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "success": false, "error": "Missing authorization header" })),
                )
            })?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "success": false, "error": "Invalid authorization format" })),
                )
            })?;

        let claims = auth::verify_token(token, &state.jwt_secret).map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "success": false, "error": "Invalid or expired token" })),
            )
        })?;

        // Verify session is still active in Redis
        let mut redis = state.auth_service.redis().clone();
        let key = format!("session:{}", claims.session_id);
        let exists: bool = redis::cmd("EXISTS")
            .arg(&key)
            .query_async(&mut redis)
            .await
            .unwrap_or(false);

        if !exists {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({ "success": false, "error": "Session expired or revoked" })),
            ));
        }

        Ok(claims)
    }
}
