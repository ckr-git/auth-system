use std::sync::Arc;
use axum::{Router, routing::{get, post, delete}, Json};
use serde_json::{json, Value};

use crate::AppState;
use crate::presentation::handler;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/health", get(health_check))
        // Subject
        .route("/api/subjects/register", post(handler::subject_handler::register))
        .route("/api/subjects/me", get(handler::auth_handler::me))
        // Credentials (auth required)
        .route("/api/credentials/totp/setup", post(handler::credential_handler::totp_setup))
        .route("/api/credentials/totp/confirm", post(handler::credential_handler::totp_confirm))
        .route("/api/credentials/totp/verify", post(handler::credential_handler::totp_verify))
        .route("/api/credentials/status", get(handler::credential_handler::credential_status))
        // Passkey (auth required)
        .route("/api/credentials/passkey/register-begin", post(handler::passkey_handler::register_begin))
        .route("/api/credentials/passkey/register-complete", post(handler::passkey_handler::register_complete))
        .route("/api/credentials/passkey/authenticate-begin", post(handler::passkey_handler::authenticate_begin))
        .route("/api/credentials/passkey/authenticate-complete", post(handler::passkey_handler::authenticate_complete))
        // Auth
        .route("/api/auth/{subject_type}/login", post(handler::auth_handler::login))
        .route("/api/auth/mfa/verify", post(handler::auth_handler::mfa_verify))
        .route("/api/auth/logout", post(handler::auth_handler::logout))
        // Passkey login (no auth required)
        .route("/api/auth/passkey/begin", post(handler::passkey_handler::login_begin))
        .route("/api/auth/passkey/complete", post(handler::passkey_handler::login_complete))
        // Sessions (auth required)
        .route("/api/sessions", get(handler::session_handler::list_sessions))
        .route("/api/sessions/{session_id}", delete(handler::session_handler::revoke_session))
        .with_state(state)
}

async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
