use async_trait::async_trait;
use kylin_common::Result;
use std::path::PathBuf;
use tokio::fs;

use crate::provider::{LayoutDescriptor, StorageProvider};

/// Local filesystem storage provider
pub struct LocalStorageProvider {
    root_path: PathBuf,
}

impl LocalStorageProvider {
    pub fn new(root_path: PathBuf) -> Self {
        Self { root_path }
    }

    fn layout_path(&self, layout: &LayoutDescriptor) -> PathBuf {
        self.root_path
            .join(&layout.dataflow_uuid)
            .join(&layout.segment_uuid)
            .join(layout.layout_id.to_string())
    }
}

#[async_trait]
impl StorageProvider for LocalStorageProvider {
    async fn write_layout_data(
        &self,
        layout: &LayoutDescriptor,
        data: Vec<u8>,
    ) -> Result<()> {
        let path = self.layout_path(layout);
        fs::create_dir_all(&path).await?;
        fs::write(path.join("data.parquet"), data).await?;
        Ok(())
    }

    async fn read_layout_data(&self, layout: &LayoutDescriptor) -> Result<Vec<u8>> {
        let path = self.layout_path(layout);
        let data = fs::read(path.join("data.parquet")).await?;
        Ok(data)
    }

    async fn delete_segment_data(
        &self,
        dataflow_uuid: &str,
        segment_uuid: &str,
    ) -> Result<()> {
        let path = self.root_path.join(dataflow_uuid).join(segment_uuid);
        if path.exists() {
            fs::remove_dir_all(path).await?;
        }
        Ok(())
    }

    async fn list_segment_layouts(
        &self,
        dataflow_uuid: &str,
        segment_uuid: &str,
    ) -> Result<Vec<i64>> {
        let path = self.root_path.join(dataflow_uuid).join(segment_uuid);
        let mut layouts = Vec::new();

        if path.exists() {
            let mut entries = fs::read_dir(&path).await?;
            while let Some(entry) = entries.next_entry().await? {
                if entry.file_type().await?.is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        if let Ok(id) = name.parse::<i64>() {
                            layouts.push(id);
                        }
                    }
                }
            }
        }

        Ok(layouts)
    }
}
