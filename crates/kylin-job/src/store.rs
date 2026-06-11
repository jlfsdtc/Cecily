use crate::job::Job;
use async_trait::async_trait;
use kylin_common::Result;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Job store trait
#[async_trait]
pub trait JobStore: Send + Sync {
    async fn create_job(&self, job: &Job) -> Result<()>;
    async fn update_job(&self, job: &Job) -> Result<()>;
    async fn get_job(&self, uuid: &str) -> Result<Option<Job>>;
    async fn list_jobs(&self, project: &str, limit: usize) -> Result<Vec<Job>>;
    async fn pending_jobs(&self, limit: usize) -> Result<Vec<Job>>;
    async fn delete_job(&self, uuid: &str) -> Result<()>;
}

/// In-memory job store implementation
pub struct InMemoryJobStore {
    jobs: RwLock<HashMap<String, Job>>,
}

impl InMemoryJobStore {
    pub fn new() -> Self {
        Self {
            jobs: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryJobStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl JobStore for InMemoryJobStore {
    async fn create_job(&self, job: &Job) -> Result<()> {
        let mut jobs = self.jobs.write().await;
        jobs.insert(job.uuid.clone(), job.clone());
        Ok(())
    }

    async fn update_job(&self, job: &Job) -> Result<()> {
        let mut jobs = self.jobs.write().await;
        jobs.insert(job.uuid.clone(), job.clone());
        Ok(())
    }

    async fn get_job(&self, uuid: &str) -> Result<Option<Job>> {
        let jobs = self.jobs.read().await;
        Ok(jobs.get(uuid).cloned())
    }

    async fn list_jobs(&self, project: &str, limit: usize) -> Result<Vec<Job>> {
        let jobs = self.jobs.read().await;
        let mut result: Vec<Job> = jobs
            .values()
            .filter(|j| j.project == project)
            .cloned()
            .collect();
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        result.truncate(limit);
        Ok(result)
    }

    async fn pending_jobs(&self, limit: usize) -> Result<Vec<Job>> {
        let jobs = self.jobs.read().await;
        let mut result: Vec<Job> = jobs
            .values()
            .filter(|j| j.status == kylin_common::JobStatus::Pending)
            .cloned()
            .collect();
        result.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        result.truncate(limit);
        Ok(result)
    }

    async fn delete_job(&self, uuid: &str) -> Result<()> {
        let mut jobs = self.jobs.write().await;
        jobs.remove(uuid);
        Ok(())
    }
}
