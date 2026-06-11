use chrono::{DateTime, Utc};
use kylin_common::JobStatus;
use serde::{Deserialize, Serialize};

/// Job type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobType {
    SegmentBuild,
    SegmentMerge,
    SegmentRefresh,
    IndexAdd,
    SnapshotBuild,
}

impl std::fmt::Display for JobType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobType::SegmentBuild => write!(f, "SEGMENT_BUILD"),
            JobType::SegmentMerge => write!(f, "SEGMENT_MERGE"),
            JobType::SegmentRefresh => write!(f, "SEGMENT_REFRESH"),
            JobType::IndexAdd => write!(f, "INDEX_ADD"),
            JobType::SnapshotBuild => write!(f, "SNAPSHOT_BUILD"),
        }
    }
}

/// Job definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub uuid: String,
    pub project: String,
    pub job_type: JobType,
    pub status: JobStatus,
    pub params: JobParams,
    pub result: Option<JobResult>,
    pub progress: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub error_message: Option<String>,
}

/// Job parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobParams {
    pub model_uuid: String,
    pub segment_id: Option<String>,
    pub time_range: Option<(i64, i64)>,
    pub layout_ids: Option<Vec<i64>>,
}

/// Job result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    pub segment_id: Option<String>,
    pub rows_affected: u64,
    pub duration_ms: u64,
}
