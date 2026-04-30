use axum::{
    extract::{FromRequest, FromRequestParts, Request, rejection::{JsonRejection, PathRejection}},
    http::{request::Parts, StatusCode},
    Json,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

pub struct AppJson<T>(pub T);

impl<S, T> FromRequest<S> for AppJson<T>
where
    axum::Json<T>: FromRequest<S, Rejection = JsonRejection>,
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = (StatusCode, Json<Value>);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                let msg = extract_readable_message(rejection.body_text());
                Err((StatusCode::BAD_REQUEST, Json(json!({ "success": false, "error": msg }))))
            }
        }
    }
}

pub struct AppPath<T>(pub T);

impl<S, T> FromRequestParts<S> for AppPath<T>
where
    axum::extract::Path<T>: FromRequestParts<S, Rejection = PathRejection>,
    S: Send + Sync,
    T: DeserializeOwned + Send,
{
    type Rejection = (StatusCode, Json<Value>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match axum::extract::Path::<T>::from_request_parts(parts, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(_) => Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "error": "Invalid path parameter" })),
            )),
        }
    }
}

fn extract_readable_message(body: String) -> String {
    if let Some(pos) = body.find(':') {
        let detail = body[pos + 1..].trim();
        if let Some(inner_pos) = detail.find(':') {
            let field_part = detail[inner_pos + 1..].trim();
            return format!("Invalid request body: {}", field_part);
        }
        return format!("Invalid request body: {}", detail);
    }
    "Invalid request body".to_string()
}
