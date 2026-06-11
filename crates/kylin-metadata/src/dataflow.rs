use kylin_common::types::PersistentEntity;
use serde::{Deserialize, Serialize};

/// A dataflow represents a materialized model instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataflow {
    #[serde(flatten)]
    pub entity: PersistentEntity,
    /// Project this dataflow belongs to
    pub project: String,
    /// Model UUID this dataflow is based on
    pub model_uuid: String,
    /// Model name (denormalized for convenience)
    pub model_name: String,
    /// Status
    pub status: DataflowStatus,
    /// Segments
    pub segments: Vec<String>,
    /// Layouts
    pub layouts: Vec<LayoutEntity>,
}

/// Dataflow status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataflowStatus {
    Active,
    Disabled,
}

impl std::fmt::Display for DataflowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataflowStatus::Active => write!(f, "ACTIVE"),
            DataflowStatus::Disabled => write!(f, "DISABLED"),
        }
    }
}

/// Layout entity represents a pre-computed aggregation index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutEntity {
    /// Layout ID (unique within a dataflow)
    pub id: i64,
    /// Dimensions (column IDs)
    pub dimensions: Vec<String>,
    /// Measures (measure IDs)
    pub measures: Vec<String>,
    /// Shard by columns
    pub shard_by_columns: Vec<String>,
    /// Whether this layout is a table index (vs aggregation index)
    pub is_table_index: bool,
    /// Storage size in bytes
    pub storage_size: u64,
    /// Row count
    pub row_count: u64,
}

/// Index plan defines which layouts to pre-compute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexPlan {
    /// Model UUID
    pub model_uuid: String,
    /// Layouts to build
    pub layouts: Vec<LayoutEntity>,
    /// Whether to auto-merge segments
    pub auto_merge: bool,
    /// Retention range in days
    pub retention_range: u32,
}
