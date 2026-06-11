use crate::job::Job;
use crate::store::JobStore;
use kylin_common::JobStatus;
use std::sync::Arc;
use tokio::time::{self, Duration};

/// Job scheduler - manages job execution
pub struct JobScheduler {
    job_store: Arc<dyn JobStore>,
    max_concurrent: usize,
}

impl JobScheduler {
    pub fn new(job_store: Arc<dyn JobStore>, max_concurrent: usize) -> Self {
        Self {
            job_store,
            max_concurrent,
        }
    }

    /// Start the scheduler loop
    pub async fn start(&self) {
        tracing::info!("Starting job scheduler with max_concurrent={}", self.max_concurrent);

        loop {
            let pending_jobs = self.job_store.pending_jobs(self.max_concurrent).await;

            match pending_jobs {
                Ok(jobs) => {
                    for job in jobs {
                        let job_store = self.job_store.clone();
                        tokio::spawn(async move {
                            Self::execute_job(job_store, job).await;
                        });
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to fetch pending jobs: {}", e);
                }
            }

            time::sleep(Duration::from_secs(1)).await;
        }
    }

    async fn execute_job(job_store: Arc<dyn JobStore>, mut job: Job) {
        tracing::info!("Executing job: {} ({})", job.uuid, job.job_type);

        job.status = JobStatus::Running;
        let _ = job_store.update_job(&job).await;

        // TODO: Execute job based on type
        let result: std::result::Result<(), anyhow::Error> = match job.job_type {
            crate::job::JobType::SegmentBuild => {
                // TODO: Execute segment build
                Ok(())
            }
            crate::job::JobType::SegmentMerge => {
                // TODO: Execute segment merge
                Ok(())
            }
            crate::job::JobType::SegmentRefresh => {
                // TODO: Execute segment refresh
                Ok(())
            }
            crate::job::JobType::IndexAdd => {
                // TODO: Execute index add
                Ok(())
            }
            crate::job::JobType::SnapshotBuild => {
                // TODO: Execute snapshot build
                Ok(())
            }
        };

        match result {
            Ok(()) => {
                job.status = JobStatus::Finished;
                job.progress = 1.0;
            }
            Err(e) => {
                job.status = JobStatus::Error;
                job.error_message = Some(e.to_string());
            }
        }

        let _ = job_store.update_job(&job).await;
    }
}
