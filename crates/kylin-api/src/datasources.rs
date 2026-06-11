use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{AppState, error::ApiError};

/// Request body for adding a datasource
#[derive(Debug, Deserialize)]
pub struct AddDatasourceRequest {
    pub name: String,
    pub source_type: String,
    pub connection_url: Option<String>,
    pub properties: Option<Value>,
}

/// Create the datasources router
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_datasources).post(add_datasource))
        .route("/{source}/tables", get(get_source_tables))
        .route("/{source}/tables/{table}", get(get_source_table))
        .route("/{source}/tables/{table}/snapshot", post(build_snapshot))
}

async fn list_datasources(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    // This is a placeholder - in a real implementation, you'd list datasources
    Ok(Json(json!({
        "code": "000",
        "data": {
            "datasources": [],
            "size": 0
        }
    })))
}

async fn add_datasource(
    State(state): State<AppState>,
    Json(body): Json<AddDatasourceRequest>,
) -> Result<Json<Value>, ApiError> {
    // Validate request
    if body.name.is_empty() {
        return Err(ApiError::BadRequest("Datasource name cannot be empty".to_string()));
    }
    if body.source_type.is_empty() {
        return Err(ApiError::BadRequest("Source type cannot be empty".to_string()));
    }

    // This is a placeholder - in a real implementation, you'd add the datasource
    Ok(Json(json!({
        "code": "000",
        "data": {
            "name": body.name,
            "source_type": body.source_type
        }
    })))
}

async fn get_source_tables(
    State(state): State<AppState>,
    Path(source): Path<String>,
) -> Result<Json<Value>, ApiError> {
    // This is a placeholder - in a real implementation, you'd get tables from the datasource
    Ok(Json(json!({
        "code": "000",
        "data": {
            "tables": [],
            "size": 0
        }
    })))
}

async fn get_source_table(
    State(state): State<AppState>,
    Path((source, table)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    // This is a placeholder - in a real implementation, you'd get table details
    Ok(Json(json!({
        "code": "000",
        "data": {
            "name": table,
            "source": source
        }
    })))
}

async fn build_snapshot(
    State(state): State<AppState>,
    Path((source, table)): Path<(String, String)>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    // This is a placeholder - in a real implementation, you'd build a snapshot
    Ok(Json(json!({
        "code": "000",
        "data": {
            "status": "submitted",
            "source": source,
            "table": table
        }
    })))
}
