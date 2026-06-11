use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use kylin_common::KylinError;
use serde_json::json;

/// API error types
#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    Internal(String),
}

impl From<KylinError> for ApiError {
    fn from(err: KylinError) -> Self {
        match err {
            KylinError::NotFound(msg) => ApiError::NotFound(msg),
            KylinError::InvalidArgument(msg) => ApiError::BadRequest(msg),
            KylinError::Internal(msg) => ApiError::Internal(msg),
            _ => ApiError::Internal(err.to_string()),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "code": status.as_str(),
            "data": null,
            "msg": message
        }));

        (status, body).into_response()
    }
}
