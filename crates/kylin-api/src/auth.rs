use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use hyper::StatusCode;

/// Authentication middleware
pub async fn auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // TODO: Implement authentication
    // For now, pass through all requests
    Ok(next.run(request).await)
}
