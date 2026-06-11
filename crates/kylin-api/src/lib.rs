pub mod models;
pub mod query;
pub mod jobs;
pub mod projects;
pub mod datasources;
pub mod auth;
pub mod error;

use axum::Router;
use kylin_metadata::MetadataStore;
use kylin_query::QueryExecutor;
use kylin_job::JobStore;
use std::sync::Arc;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub metadata_store: Arc<dyn MetadataStore>,
    pub query_executor: Arc<QueryExecutor>,
    pub job_store: Arc<dyn JobStore>,
}

/// Create the API router
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .nest("/api/models", models::router())
        .nest("/api/query", query::router())
        .nest("/api/jobs", jobs::router())
        .nest("/api/projects", projects::router())
        .nest("/api/datasources", datasources::router())
        .with_state(state)
}
