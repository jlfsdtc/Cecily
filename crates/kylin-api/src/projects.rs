use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{AppState, error::ApiError};
use kylin_common::types::PersistentEntity;
use kylin_metadata::project::Project;

/// Request body for creating a project
#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub default_database: Option<String>,
}

/// Request body for updating a project
#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub description: Option<String>,
    pub default_database: Option<String>,
}

/// Create the projects router
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_projects).post(create_project))
        .route("/{project}", get(get_project).put(update_project).delete(delete_project))
}

async fn list_projects(
    State(state): State<AppState>,
) -> Result<Json<Value>, ApiError> {
    let projects = state.metadata_store.list_projects().await?;

    Ok(Json(json!({
        "code": "000",
        "data": {
            "projects": projects,
            "size": projects.len()
        }
    })))
}

async fn create_project(
    State(state): State<AppState>,
    Json(body): Json<CreateProjectRequest>,
) -> Result<Json<Value>, ApiError> {
    // Validate request
    if body.name.is_empty() {
        return Err(ApiError::BadRequest("Project name cannot be empty".to_string()));
    }

    // Check if project already exists
    if state.metadata_store.load_project(&body.name).await.is_ok() {
        return Err(ApiError::BadRequest(format!("Project already exists: {}", body.name)));
    }

    // Create project
    let mut project = Project::new(&body.name);
    project.description = body.description;
    project.default_database = body.default_database;

    state.metadata_store.save_project(&project).await?;

    Ok(Json(json!({
        "code": "000",
        "data": project
    })))
}

async fn get_project(
    State(state): State<AppState>,
    Path(project_name): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let project = state.metadata_store.load_project(&project_name).await?;

    Ok(Json(json!({
        "code": "000",
        "data": project
    })))
}

async fn update_project(
    State(state): State<AppState>,
    Path(project_name): Path<String>,
    Json(body): Json<UpdateProjectRequest>,
) -> Result<Json<Value>, ApiError> {
    // Load existing project
    let mut project = state.metadata_store.load_project(&project_name).await?;

    // Update fields
    if let Some(description) = body.description {
        project.description = Some(description);
    }
    if let Some(default_database) = body.default_database {
        project.default_database = Some(default_database);
    }

    state.metadata_store.save_project(&project).await?;

    Ok(Json(json!({
        "code": "000",
        "data": project
    })))
}

async fn delete_project(
    State(state): State<AppState>,
    Path(project_name): Path<String>,
) -> Result<Json<Value>, ApiError> {
    state.metadata_store.delete_project(&project_name).await?;

    Ok(Json(json!({
        "code": "000",
        "data": {}
    })))
}
