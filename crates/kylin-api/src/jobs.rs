use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{AppState, error::ApiError};
use kylin_common::JobStatus;
use kylin_job::{Job, JobType, JobParams};

/// Query parameters for listing jobs
#[derive(Debug, Deserialize)]
pub struct ListJobsParams {
    pub project: String,
    pub limit: Option<usize>,
}

/// Request body for creating a job
#[derive(Debug, Deserialize)]
pub struct CreateJobRequest {
    pub project: String,
    pub job_type: String,
    pub model_uuid: String,
    pub segment_id: Option<String>,
    pub time_range: Option<(i64, i64)>,
    pub layout_ids: Option<Vec<i64>>,
}

/// Create the jobs router
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_jobs).post(create_job))
        .route("/{job}", get(get_job).put(update_job).delete(delete_job))
        .route("/{job}/status", get(get_job_status))
}

async fn list_jobs(
    State(state): State<AppState>,
    Query(params): Query<ListJobsParams>,
) -> Result<Json<Value>, ApiError> {
    let limit = params.limit.unwrap_or(100);
    let jobs = state.job_store.list_jobs(&params.project, limit).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(json!({
        "code": "000",
        "data": {
            "jobs": jobs,
            "size": jobs.len()
        }
    })))
}

async fn create_job(
    State(state): State<AppState>,
    Json(body): Json<CreateJobRequest>,
) -> Result<Json<Value>, ApiError> {
    // Validate request
    if body.model_uuid.is_empty() {
        return Err(ApiError::BadRequest("Model UUID cannot be empty".to_string()));
    }

    // Parse job type
    let job_type = match body.job_type.as_str() {
        "SEGMENT_BUILD" => JobType::SegmentBuild,
        "SEGMENT_MERGE" => JobType::SegmentMerge,
        "SEGMENT_REFRESH" => JobType::SegmentRefresh,
        "INDEX_ADD" => JobType::IndexAdd,
        "SNAPSHOT_BUILD" => JobType::SnapshotBuild,
        other => return Err(ApiError::BadRequest(format!("Invalid job type: {}", other))),
    };

    // Create job
    let job = Job {
        uuid: uuid::Uuid::new_v4().to_string(),
        project: body.project,
        job_type,
        status: JobStatus::Pending,
        params: JobParams {
            model_uuid: body.model_uuid,
            segment_id: body.segment_id,
            time_range: body.time_range,
            layout_ids: body.layout_ids,
        },
        result: None,
        progress: 0.0,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        error_message: None,
    };

    state.job_store.create_job(&job).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(json!({
        "code": "000",
        "data": job
    })))
}

async fn get_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let job = state.job_store.get_job(&job_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    match job {
        Some(job) => Ok(Json(json!({
            "code": "000",
            "data": job
        }))),
        None => Err(ApiError::NotFound(format!("Job not found: {}", job_id))),
    }
}

async fn update_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    // This is a placeholder - in a real implementation, you'd update the job
    Ok(Json(json!({
        "code": "000",
        "data": {}
    })))
}

async fn delete_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    state.job_store.delete_job(&job_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(json!({
        "code": "000",
        "data": {}
    })))
}

async fn get_job_status(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let job = state.job_store.get_job(&job_id).await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    match job {
        Some(job) => Ok(Json(json!({
            "code": "000",
            "data": {
                "uuid": job.uuid,
                "status": job.status,
                "progress": job.progress,
                "error_message": job.error_message
            }
        }))),
        None => Err(ApiError::NotFound(format!("Job not found: {}", job_id))),
    }
}
