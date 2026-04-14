mod domain;
mod application;
mod infrastructure;
mod presentation;

use std::sync::Arc;
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use webauthn_rs::prelude::*;

use application::service::{AuthService, CredentialService, SessionService, PasskeyService};
use infrastructure::persistence::{PgSubjectRepository, PgCredentialRepository, PgSessionRepository};

pub type AppAuthService = AuthService<PgSubjectRepository, PgCredentialRepository, PgSessionRepository>;
pub type AppCredentialService = CredentialService<PgCredentialRepository>;
pub type AppSessionService = SessionService<PgSessionRepository>;
pub type AppPasskeyService = PasskeyService<PgCredentialRepository, PgSubjectRepository>;

pub struct AppState {
    pub auth_service: AppAuthService,
    pub credential_service: AppCredentialService,
    pub session_service: AppSessionService,
    pub passkey_service: AppPasskeyService,
    pub jwt_secret: String,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "auth_system=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set");

    let rp_id = std::env::var("WEBAUTHN_RP_ID")
        .unwrap_or_else(|_| "localhost".to_string());
    let rp_origin = std::env::var("WEBAUTHN_RP_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:5173".to_string());

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Failed to run migrations");

    let redis_client = redis::Client::open(redis_url)
        .expect("Failed to create Redis client");
    let redis_conn = redis::aio::ConnectionManager::new(redis_client)
        .await
        .expect("Failed to connect to Redis");

    let rp_origin_url = url::Url::parse(&rp_origin).expect("Invalid WEBAUTHN_RP_ORIGIN");
    let webauthn = WebauthnBuilder::new(&rp_id, &rp_origin_url)
        .expect("Failed to create WebauthnBuilder")
        .rp_name("Auth System")
        .build()
        .expect("Failed to build Webauthn");

    let state = Arc::new(AppState {
        auth_service: AuthService::new(
            PgSubjectRepository::new(db.clone()),
            PgCredentialRepository::new(db.clone()),
            PgSessionRepository::new(db.clone()),
            redis_conn.clone(),
            jwt_secret.clone(),
        ),
        credential_service: CredentialService::new(
            PgCredentialRepository::new(db.clone()),
            redis_conn.clone(),
        ),
        session_service: SessionService::new(
            PgSessionRepository::new(db.clone()),
            redis_conn.clone(),
        ),
        passkey_service: PasskeyService::new(
            PgCredentialRepository::new(db.clone()),
            PgSubjectRepository::new(db.clone()),
            webauthn,
            redis_conn.clone(),
        ),
        jwt_secret,
    });

    let cors = CorsLayer::new()
        .allow_origin([
            "http://localhost:5173".parse::<http::HeaderValue>().unwrap(),
        ])
        .allow_methods([
            http::Method::GET,
            http::Method::POST,
            http::Method::PUT,
            http::Method::DELETE,
            http::Method::OPTIONS,
        ])
        .allow_headers([
            http::header::CONTENT_TYPE,
            http::header::AUTHORIZATION,
            http::HeaderName::from_static("x-device-name"),
        ])
        .allow_credentials(true);

    let app = Router::new()
        .merge(presentation::router::routes(state.clone()))
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr = "0.0.0.0:3000";
    tracing::info!("Server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
