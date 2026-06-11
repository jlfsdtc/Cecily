use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{AppState, error::ApiError};
use kylin_common::types::{ModelType, PersistentEntity};
use kylin_metadata::model::DataModel;

/// Query parameters for listing models
#[derive(Debug, Deserialize)]
pub struct ListModelsParams {
    pub project: String,
}

/// Request body for creating a model
#[derive(Debug, Deserialize)]
pub struct CreateModelRequest {
    pub project: String,
    pub name: String,
    pub root_fact_table: String,
    pub model_type: Option<String>,
}

/// Request body for updating a model
#[derive(Debug, Deserialize)]
pub struct UpdateModelRequest {
    pub name: Option<String>,
    pub root_fact_table: Option<String>,
    pub model_type: Option<String>,
    pub filter_condition: Option<String>,
}

/// Create the models router
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_models).post(create_model))
        .route("/{model}", get(get_model).put(update_model).delete(delete_model))
        .route("/{model}/name", put(rename_model))
        .route("/{model}/status", put(toggle_model_status))
        .route("/{model}/clone", post(clone_model))
        .route("/validate_model", post(validate_model))
}

async fn list_models(
    State(state): State<AppState>,
    Query(params): Query<ListModelsParams>,
) -> Result<Json<Value>, ApiError> {
    let models = state.metadata_store.list_models(&params.project).await?;

    Ok(Json(json!({
        "code": "000",
        "data": {
            "models": models,
            "size": models.len()
        }
    })))
}

async fn create_model(
    State(state): State<AppState>,
    Json(body): Json<CreateModelRequest>,
) -> Result<Json<Value>, ApiError> {
    // Parse model type
    let model_type = match body.model_type.as_deref() {
        Some("BATCH") | None => ModelType::Batch,
        Some("STREAMING") => ModelType::Streaming,
        Some("HYBRID") => ModelType::Hybrid,
        Some(other) => return Err(ApiError::BadRequest(format!("Invalid model type: {}", other))),
    };

    // Create model
    let model = DataModel {
        entity: PersistentEntity::new(),
        name: body.name,
        root_fact_table: body.root_fact_table,
        model_type,
        join_tables: vec![],
        all_columns: vec![],
        all_measures: vec![],
        filter_condition: None,
        partition_desc: None,
        computed_columns: vec![],
    };

    // Ensure project exists
    if state.metadata_store.load_project(&body.project).await.is_err() {
        return Err(ApiError::NotFound(format!("Project not found: {}", body.project)));
    }

    // Save model
    state.metadata_store.save_model(&body.project, &model).await?;

    Ok(Json(json!({
        "code": "000",
        "data": model
    })))
}

async fn get_model(
    State(state): State<AppState>,
    Path(model_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    // Try to load model from all projects
    let projects = state.metadata_store.list_projects().await?;

    for project in projects {
        if let Ok(model) = state.metadata_store.load_model(&project.name, &model_id).await {
            return Ok(Json(json!({
                "code": "000",
                "data": model
            })));
        }
    }

    Err(ApiError::NotFound(format!("Model not found: {}", model_id)))
}

async fn update_model(
    State(state): State<AppState>,
    Path(model_id): Path<String>,
    Json(body): Json<UpdateModelRequest>,
) -> Result<Json<Value>, ApiError> {
    // Find the model
    let projects = state.metadata_store.list_projects().await?;
    let mut found_model = None;
    let mut found_project = None;

    for project in projects {
        if let Ok(model) = state.metadata_store.load_model(&project.name, &model_id).await {
            found_model = Some(model);
            found_project = Some(project.name);
            break;
        }
    }

    let (mut model, project) = match (found_model, found_project) {
        (Some(m), Some(p)) => (m, p),
        _ => return Err(ApiError::NotFound(format!("Model not found: {}", model_id))),
    };

    // Update fields
    if let Some(name) = body.name {
        model.name = name;
    }
    if let Some(root_fact_table) = body.root_fact_table {
        model.root_fact_table = root_fact_table;
    }
    if let Some(model_type) = body.model_type {
        model.model_type = match model_type.as_str() {
            "BATCH" => ModelType::Batch,
            "STREAMING" => ModelType::Streaming,
            "HYBRID" => ModelType::Hybrid,
            _ => return Err(ApiError::BadRequest(format!("Invalid model type: {}", model_type))),
        };
    }
    if let Some(filter_condition) = body.filter_condition {
        model.filter_condition = Some(filter_condition);
    }

    // Save updated model
    state.metadata_store.save_model(&project, &model).await?;

    Ok(Json(json!({
        "code": "000",
        "data": model
    })))
}

async fn delete_model(
    State(state): State<AppState>,
    Path(model_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    // Find and delete the model
    let projects = state.metadata_store.list_projects().await?;

    for project in projects {
        if state.metadata_store.delete_model(&project.name, &model_id).await.is_ok() {
            return Ok(Json(json!({
                "code": "000",
                "data": {}
            })));
        }
    }

    Err(ApiError::NotFound(format!("Model not found: {}", model_id)))
}

async fn rename_model(
    State(state): State<AppState>,
    Path(model_id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let new_name = body.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::BadRequest("Missing 'name' field".to_string()))?;

    // Find the model
    let projects = state.metadata_store.list_projects().await?;
    let mut found_model = None;
    let mut found_project = None;

    for project in projects {
        if let Ok(model) = state.metadata_store.load_model(&project.name, &model_id).await {
            found_model = Some(model);
            found_project = Some(project.name);
            break;
        }
    }

    let (mut model, project) = match (found_model, found_project) {
        (Some(m), Some(p)) => (m, p),
        _ => return Err(ApiError::NotFound(format!("Model not found: {}", model_id))),
    };

    model.name = new_name.to_string();
    state.metadata_store.save_model(&project, &model).await?;

    Ok(Json(json!({
        "code": "000",
        "data": model
    })))
}

async fn toggle_model_status(
    State(state): State<AppState>,
    Path(model_id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    // This is a placeholder - in a real implementation, you'd toggle the model status
    Ok(Json(json!({
        "code": "000",
        "data": {}
    })))
}

async fn clone_model(
    State(state): State<AppState>,
    Path(model_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    // Find the model
    let projects = state.metadata_store.list_projects().await?;
    let mut found_model = None;
    let mut found_project = None;

    for project in projects {
        if let Ok(model) = state.metadata_store.load_model(&project.name, &model_id).await {
            found_model = Some(model);
            found_project = Some(project.name);
            break;
        }
    }

    let (model, project) = match (found_model, found_project) {
        (Some(m), Some(p)) => (m, p),
        _ => return Err(ApiError::NotFound(format!("Model not found: {}", model_id))),
    };

    // Create a clone
    let mut cloned = model.clone();
    cloned.entity = PersistentEntity::new();
    cloned.name = format!("{}_clone", model.name);

    state.metadata_store.save_model(&project, &cloned).await?;

    Ok(Json(json!({
        "code": "000",
        "data": cloned
    })))
}

async fn validate_model(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    // This is a placeholder - in a real implementation, you'd validate the model
    Ok(Json(json!({
        "code": "000",
        "data": {
            "valid": true
        }
    })))
}
