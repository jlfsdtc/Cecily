use kylin_common::JobStatus;
use kylin_job::{Job, JobType, JobParams, JobResult, InMemoryJobStore, JobStore};
use chrono::Utc;

fn create_test_job(project: &str, job_type: JobType) -> Job {
    Job {
        uuid: uuid::Uuid::new_v4().to_string(),
        project: project.to_string(),
        job_type,
        status: JobStatus::Pending,
        params: JobParams {
            model_uuid: "test-model-uuid".to_string(),
            segment_id: Some("test-segment".to_string()),
            time_range: Some((1000000, 2000000)),
            layout_ids: Some(vec![1, 2, 3]),
        },
        result: None,
        progress: 0.0,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        error_message: None,
    }
}

#[tokio::test]
async fn test_create_and_get_job() {
    let store = InMemoryJobStore::new();
    let job = create_test_job("test_project", JobType::SegmentBuild);

    // Create job
    store.create_job(&job).await.unwrap();

    // Get job
    let retrieved = store.get_job(&job.uuid).await.unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.uuid, job.uuid);
    assert_eq!(retrieved.project, "test_project");
    assert_eq!(retrieved.job_type, JobType::SegmentBuild);
    assert_eq!(retrieved.status, JobStatus::Pending);
}

#[tokio::test]
async fn test_update_job() {
    let store = InMemoryJobStore::new();
    let mut job = create_test_job("test_project", JobType::SegmentBuild);

    // Create job
    store.create_job(&job).await.unwrap();

    // Update job status
    job.status = JobStatus::Running;
    job.progress = 0.5;
    store.update_job(&job).await.unwrap();

    // Verify update
    let retrieved = store.get_job(&job.uuid).await.unwrap().unwrap();
    assert_eq!(retrieved.status, JobStatus::Running);
    assert_eq!(retrieved.progress, 0.5);
}

#[tokio::test]
async fn test_list_jobs() {
    let store = InMemoryJobStore::new();

    // Create multiple jobs
    let job1 = create_test_job("project1", JobType::SegmentBuild);
    let job2 = create_test_job("project1", JobType::SegmentMerge);
    let job3 = create_test_job("project2", JobType::SegmentBuild);

    store.create_job(&job1).await.unwrap();
    store.create_job(&job2).await.unwrap();
    store.create_job(&job3).await.unwrap();

    // List jobs for project1
    let jobs = store.list_jobs("project1", 10).await.unwrap();
    assert_eq!(jobs.len(), 2);

    // List jobs for project2
    let jobs = store.list_jobs("project2", 10).await.unwrap();
    assert_eq!(jobs.len(), 1);

    // List with limit
    let jobs = store.list_jobs("project1", 1).await.unwrap();
    assert_eq!(jobs.len(), 1);
}

#[tokio::test]
async fn test_pending_jobs() {
    let store = InMemoryJobStore::new();

    // Create jobs with different statuses
    let mut job1 = create_test_job("project1", JobType::SegmentBuild);
    job1.status = JobStatus::Pending;

    let mut job2 = create_test_job("project1", JobType::SegmentMerge);
    job2.status = JobStatus::Running;

    let mut job3 = create_test_job("project1", JobType::SegmentRefresh);
    job3.status = JobStatus::Pending;

    store.create_job(&job1).await.unwrap();
    store.create_job(&job2).await.unwrap();
    store.create_job(&job3).await.unwrap();

    // Get pending jobs
    let pending = store.pending_jobs(10).await.unwrap();
    assert_eq!(pending.len(), 2);
    assert!(pending.iter().all(|j| j.status == JobStatus::Pending));
}

#[tokio::test]
async fn test_delete_job() {
    let store = InMemoryJobStore::new();
    let job = create_test_job("test_project", JobType::SegmentBuild);

    // Create job
    store.create_job(&job).await.unwrap();

    // Verify it exists
    assert!(store.get_job(&job.uuid).await.unwrap().is_some());

    // Delete job
    store.delete_job(&job.uuid).await.unwrap();

    // Verify it's gone
    assert!(store.get_job(&job.uuid).await.unwrap().is_none());
}

#[tokio::test]
async fn test_job_not_found() {
    let store = InMemoryJobStore::new();

    // Try to get non-existent job
    let result = store.get_job("non-existent-uuid").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_multiple_job_types() {
    let store = InMemoryJobStore::new();

    // Create jobs of different types
    let job1 = create_test_job("project1", JobType::SegmentBuild);
    let job2 = create_test_job("project1", JobType::SegmentMerge);
    let job3 = create_test_job("project1", JobType::SegmentRefresh);
    let job4 = create_test_job("project1", JobType::IndexAdd);
    let job5 = create_test_job("project1", JobType::SnapshotBuild);

    store.create_job(&job1).await.unwrap();
    store.create_job(&job2).await.unwrap();
    store.create_job(&job3).await.unwrap();
    store.create_job(&job4).await.unwrap();
    store.create_job(&job5).await.unwrap();

    // List all jobs
    let jobs = store.list_jobs("project1", 10).await.unwrap();
    assert_eq!(jobs.len(), 5);
}

#[tokio::test]
async fn test_job_with_result() {
    let store = InMemoryJobStore::new();
    let mut job = create_test_job("test_project", JobType::SegmentBuild);

    // Create job
    store.create_job(&job).await.unwrap();

    // Update job with result
    job.status = JobStatus::Finished;
    job.progress = 1.0;
    job.result = Some(JobResult {
        segment_id: Some("new-segment".to_string()),
        rows_affected: 1000,
        duration_ms: 5000,
    });
    store.update_job(&job).await.unwrap();

    // Verify result
    let retrieved = store.get_job(&job.uuid).await.unwrap().unwrap();
    assert_eq!(retrieved.status, JobStatus::Finished);
    assert_eq!(retrieved.progress, 1.0);
    assert!(retrieved.result.is_some());
    let result = retrieved.result.unwrap();
    assert_eq!(result.rows_affected, 1000);
    assert_eq!(result.duration_ms, 5000);
}

#[tokio::test]
async fn test_job_with_error() {
    let store = InMemoryJobStore::new();
    let mut job = create_test_job("test_project", JobType::SegmentBuild);

    // Create job
    store.create_job(&job).await.unwrap();

    // Update job with error
    job.status = JobStatus::Error;
    job.error_message = Some("Test error message".to_string());
    store.update_job(&job).await.unwrap();

    // Verify error
    let retrieved = store.get_job(&job.uuid).await.unwrap().unwrap();
    assert_eq!(retrieved.status, JobStatus::Error);
    assert_eq!(retrieved.error_message, Some("Test error message".to_string()));
}
