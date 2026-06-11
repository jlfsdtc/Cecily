use kylin_common::types::{PersistentEntity, SegmentStatus};
use serde::{Deserialize, Serialize};

/// A segment represents a time range of data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    #[serde(flatten)]
    pub entity: PersistentEntity,
    /// Segment name (usually the time range)
    pub name: String,
    /// Dataflow UUID this segment belongs to
    pub dataflow_uuid: String,
    /// Segment status
    pub status: SegmentStatus,
    /// Time range start (timestamp millis)
    pub time_range_start: i64,
    /// Time range end (timestamp millis)
    pub time_range_end: i64,
    /// Source count (rows in source)
    pub source_count: u64,
    /// Size in bytes
    pub size_bytes: u64,
}

impl Segment {
    /// Create a new segment
    pub fn new(
        dataflow_uuid: &str,
        time_range_start: i64,
        time_range_end: i64,
    ) -> Self {
        Self {
            entity: PersistentEntity::new(),
            name: format!("{}_{}", time_range_start, time_range_end),
            dataflow_uuid: dataflow_uuid.to_string(),
            status: SegmentStatus::Loading,
            time_range_start,
            time_range_end,
            source_count: 0,
            size_bytes: 0,
        }
    }
}
