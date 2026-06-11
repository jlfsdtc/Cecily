use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{AppState, error::ApiError};

/// Request body for executing a query
#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub sql: String,
    pub project: String,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

/// Create the query router
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(execute_query))
}

async fn execute_query(
    State(state): State<AppState>,
    Json(body): Json<QueryRequest>,
) -> Result<Json<Value>, ApiError> {
    // Validate request
    if body.sql.is_empty() {
        return Err(ApiError::BadRequest("SQL query cannot be empty".to_string()));
    }
    if body.project.is_empty() {
        return Err(ApiError::BadRequest("Project cannot be empty".to_string()));
    }

    // Execute query
    let result = state.query_executor
        .execute(&body.sql, &body.project)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(json!({
        "code": "000",
        "data": result.to_json()
    })))
}
