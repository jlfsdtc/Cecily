use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Base entity with common fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentEntity {
    pub uuid: String,
    pub last_modified: i64,
    pub version: i64,
}

impl PersistentEntity {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            last_modified: Utc::now().timestamp_millis(),
            version: 1,
        }
    }
}

impl Default for PersistentEntity {
    fn default() -> Self {
        Self::new()
    }
}

/// Data model type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelType {
    Batch,
    Streaming,
    Hybrid,
}

impl std::fmt::Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelType::Batch => write!(f, "BATCH"),
            ModelType::Streaming => write!(f, "STREAMING"),
            ModelType::Hybrid => write!(f, "HYBRID"),
        }
    }
}

/// Data storage type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataStorageType {
    V1,
    Delta,
    Iceberg,
}

impl std::fmt::Display for DataStorageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataStorageType::V1 => write!(f, "V1"),
            DataStorageType::Delta => write!(f, "DELTA"),
            DataStorageType::Iceberg => write!(f, "ICEBERG"),
        }
    }
}

/// Job status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Pending,
    Running,
    Finished,
    Error,
    Killed,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Pending => write!(f, "PENDING"),
            JobStatus::Running => write!(f, "RUNNING"),
            JobStatus::Finished => write!(f, "FINISHED"),
            JobStatus::Error => write!(f, "ERROR"),
            JobStatus::Killed => write!(f, "KILLED"),
        }
    }
}

/// Segment status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SegmentStatus {
    Ready,
    Loading,
    Refreshing,
    Merging,
}

impl std::fmt::Display for SegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SegmentStatus::Ready => write!(f, "READY"),
            SegmentStatus::Loading => write!(f, "LOADING"),
            SegmentStatus::Refreshing => write!(f, "REFRESHING"),
            SegmentStatus::Merging => write!(f, "MERGING"),
        }
    }
}

/// Column data type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KylinDataType {
    TinyInt,
    SmallInt,
    Int,
    BigInt,
    Float,
    Double,
    Decimal { precision: u8, scale: u8 },
    Boolean,
    String,
    Varchar { max_length: u32 },
    Char { length: u32 },
    Date,
    Timestamp,
    Binary,
}

impl std::fmt::Display for KylinDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KylinDataType::TinyInt => write!(f, "tinyint"),
            KylinDataType::SmallInt => write!(f, "smallint"),
            KylinDataType::Int => write!(f, "int"),
            KylinDataType::BigInt => write!(f, "bigint"),
            KylinDataType::Float => write!(f, "float"),
            KylinDataType::Double => write!(f, "double"),
            KylinDataType::Decimal { precision, scale } => {
                write!(f, "decimal({},{})", precision, scale)
            }
            KylinDataType::Boolean => write!(f, "boolean"),
            KylinDataType::String => write!(f, "string"),
            KylinDataType::Varchar { max_length } => write!(f, "varchar({})", max_length),
            KylinDataType::Char { length } => write!(f, "char({})", length),
            KylinDataType::Date => write!(f, "date"),
            KylinDataType::Timestamp => write!(f, "timestamp"),
            KylinDataType::Binary => write!(f, "binary"),
        }
    }
}
