use serde::{Deserialize, Serialize};

/// Layout data file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutFileMeta {
    pub layout_id: i64,
    pub dataflow_uuid: String,
    pub segment_uuid: String,
    pub file_path: String,
    pub file_size: u64,
    pub row_count: u64,
}
