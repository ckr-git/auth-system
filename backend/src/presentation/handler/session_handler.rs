use std::sync::Arc;
use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::AppState;
use crate::application::dto::SessionInfo;
use crate::infrastructure::auth::Claims;
use crate::presentation::rejection::AppPath;
use super::subject_handler::map_domain_error;

pub async fn list_sessions(
    claims: Claims,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let sessions = state.session_service.list_active(claims.sub).await.map_err(map_domain_error)?;

    let data: Vec<SessionInfo> = sessions.iter().map(|s| SessionInfo {
        session_id: s.id,
        device_name: s.device_name.clone(),
        device_ip: s.device_ip.clone(),
        created_at: s.created_at,
        last_active_at: s.last_active_at,
        is_current: s.id == claims.session_id,
    }).collect();

    Ok(Json(json!({ "success": true, "data": data })))
}

pub async fn revoke_session(
    claims: Claims,
    AppPath(session_id): AppPath<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    state.session_service.revoke(session_id, claims.sub).await.map_err(map_domain_error)?;
    Ok(Json(json!({ "success": true, "data": null })))
}
