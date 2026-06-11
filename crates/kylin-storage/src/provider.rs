use async_trait::async_trait;
use kylin_common::Result;
use kylin_metadata::segment::Segment;
use serde::{Deserialize, Serialize};

/// Storage layout descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutDescriptor {
    pub dataflow_uuid: String,
    pub layout_id: i64,
    pub segment_uuid: String,
}

/// Storage provider trait
#[async_trait]
pub trait StorageProvider: Send + Sync {
    /// Write data for a layout segment
    async fn write_layout_data(
        &self,
        layout: &LayoutDescriptor,
        data: Vec<u8>,
    ) -> Result<()>;

    /// Read data for a layout segment
    async fn read_layout_data(&self, layout: &LayoutDescriptor) -> Result<Vec<u8>>;

    /// Delete data for a segment
    async fn delete_segment_data(
        &self,
        dataflow_uuid: &str,
        segment_uuid: &str,
    ) -> Result<()>;

    /// List all layout files for a segment
    async fn list_segment_layouts(
        &self,
        dataflow_uuid: &str,
        segment_uuid: &str,
    ) -> Result<Vec<i64>>;
}
