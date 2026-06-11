use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

use kylin_api::{AppState, create_router};
use kylin_metadata::{SqliteMetadataStore, MetadataManager};
use kylin_job::InMemoryJobStore;
use kylin_query::QueryExecutor;
use kylin_storage::LocalStorageProvider;
use std::sync::Arc;

async fn create_test_app() -> (Router, MetadataManager) {
    let store = SqliteMetadataStore::new_in_memory().await.unwrap();
    store.run_migrations().await.unwrap();

    let manager = MetadataManager::new(Arc::new(store));
    let storage = Arc::new(LocalStorageProvider::new(std::env::temp_dir().join("kylin_test")));
    let query_executor = Arc::new(QueryExecutor::new(manager.store().clone(), storage));
    let job_store = Arc::new(InMemoryJobStore::new());

    let state = AppState {
        metadata_store: manager.store().clone(),
        query_executor,
        job_store,
    };

    let app = create_router(state);
    (app, manager)
}

#[tokio::test]
async fn test_list_projects_empty() {
    let (app, _) = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/projects")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], "000");
    assert_eq!(json["data"]["size"], 0);
}

#[tokio::test]
async fn test_create_project() {
    let (app, manager) = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/projects")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "name": "test_project",
                        "description": "Test project"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], "000");
    assert_eq!(json["data"]["name"], "test_project");
}

#[tokio::test]
async fn test_get_project() {
    let (app, manager) = create_test_app().await;

    // Create project first
    manager.create_project("test_project", Some("Test")).await.unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/projects/test_project")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], "000");
    assert_eq!(json["data"]["name"], "test_project");
}

#[tokio::test]
async fn test_list_models_empty() {
    let (app, manager) = create_test_app().await;

    // Create project first
    manager.create_project("test_project", Some("Test")).await.unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/models?project=test_project")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], "000");
    assert_eq!(json["data"]["size"], 0);
}

#[tokio::test]
async fn test_create_model() {
    let (app, manager) = create_test_app().await;

    // Create project first
    manager.create_project("test_project", Some("Test")).await.unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/models")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "project": "test_project",
                        "name": "test_model",
                        "root_fact_table": "DEFAULT.SALES"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], "000");
    assert_eq!(json["data"]["name"], "test_model");
}

#[tokio::test]
async fn test_query_validation() {
    let (app, manager) = create_test_app().await;

    // Create project first
    manager.create_project("test_project", Some("Test")).await.unwrap();

    // Test empty SQL
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/query")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "sql": "",
                        "project": "test_project"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_job_creation() {
    let (app, manager) = create_test_app().await;

    // Create project first
    manager.create_project("test_project", Some("Test")).await.unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/jobs")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "project": "test_project",
                        "job_type": "SEGMENT_BUILD",
                        "model_uuid": "test-model-uuid"
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["code"], "000");
    // Job type is serialized as the enum variant name
    assert!(json["data"]["job_type"].is_string());
}

#[tokio::test]
async fn test_not_found() {
    let (app, _) = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/projects/non_existent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
