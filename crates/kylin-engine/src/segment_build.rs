use crate::flat_table::FlatTableBuilder;
use crate::layout_build::LayoutBuilder;
use crate::parquet_writer::ParquetWriter;
use arrow::record_batch::RecordBatch;
use kylin_common::Result;
use kylin_metadata::{DataModel, Segment, SegmentStatus, MetadataStore};
use kylin_metadata::dataflow::Dataflow;
use std::path::PathBuf;
use std::sync::Arc;

/// Segment build job - builds index layouts for a data segment
pub struct SegmentBuildJob {
    model: DataModel,
    dataflow: Dataflow,
    segment: Segment,
    metadata_store: Arc<dyn MetadataStore>,
    data_dir: PathBuf,
    progress: f32,
}

impl SegmentBuildJob {
    /// Create a new segment build job
    pub fn new(
        model: DataModel,
        dataflow: Dataflow,
        segment: Segment,
        metadata_store: Arc<dyn MetadataStore>,
        data_dir: PathBuf,
    ) -> Self {
        Self {
            model,
            dataflow,
            segment,
            metadata_store,
            data_dir,
            progress: 0.0,
        }
    }

    /// Execute the build job
    pub async fn execute(&mut self) -> Result<()> {
        tracing::info!("Starting segment build for model: {}", self.model.name);

        // Stage 1: Build flat table
        self.update_progress(0.1).await?;
        let flat_table = self.build_flat_table().await?;

        // Stage 2: Compute layouts
        self.update_progress(0.3).await?;
        self.compute_layouts(&flat_table).await?;

        // Stage 3: Finalize
        self.update_progress(1.0).await?;
        self.finalize().await?;

        tracing::info!("Segment build completed for model: {}", self.model.name);
        Ok(())
    }

    /// Build flat table by joining fact and lookup tables
    async fn build_flat_table(&self) -> Result<RecordBatch> {
        tracing::info!("Building flat table for model: {}", self.model.name);

        let builder = FlatTableBuilder::new(self.model.clone());
        let flat_table = builder.build().await?;

        tracing::info!(
            "Flat table built with {} rows, {} columns",
            flat_table.num_rows(),
            flat_table.num_columns()
        );

        Ok(flat_table)
    }

    /// Compute all layouts for this segment
    async fn compute_layouts(&mut self, flat_table: &RecordBatch) -> Result<()> {
        let layouts = self.dataflow.layouts.clone();
        let total_layouts = layouts.len();
        tracing::info!("Computing {} layouts", total_layouts);

        for (idx, layout) in layouts.iter().enumerate() {
            let progress = 0.3 + (0.6 * (idx as f32 / total_layouts as f32));
            self.progress = progress;
            tracing::info!("Build progress: {:.0}%", progress * 100.0);

            tracing::info!("Building layout {}/{}: {}", idx + 1, total_layouts, layout.id);

            let builder = LayoutBuilder::new(self.model.clone(), layout.clone());
            let layout_data = builder.build(flat_table).await?;

            // Write layout data to Parquet
            let layout_dir = self.layout_dir(&layout.id);
            std::fs::create_dir_all(&layout_dir)
                .map_err(|e| kylin_common::KylinError::Engine(format!("Failed to create directory: {}", e)))?;

            let parquet_path = layout_dir.join("data.parquet");
            let writer = ParquetWriter::new();
            writer.write_batch(&parquet_path, &layout_data)?;

            tracing::info!(
                "Layout {} written to {:?} with {} rows",
                layout.id,
                parquet_path,
                layout_data.num_rows()
            );
        }

        Ok(())
    }

    /// Finalize the segment build
    async fn finalize(&mut self) -> Result<()> {
        tracing::info!("Finalizing segment build");

        // Update segment status
        self.segment.status = SegmentStatus::Ready;
        self.segment.source_count = 0; // Would be set from actual data
        self.segment.size_bytes = self.calculate_segment_size()?;

        // Save segment metadata
        self.metadata_store.save_segment(&self.segment).await?;

        tracing::info!("Segment build finalized");
        Ok(())
    }

    /// Get the directory for a segment
    fn segment_dir(&self) -> PathBuf {
        self.data_dir
            .join(&self.dataflow.project)
            .join(&self.dataflow.model_uuid)
            .join(&self.segment.entity.uuid)
    }

    /// Get the directory for a layout
    fn layout_dir(&self, layout_id: &i64) -> PathBuf {
        self.segment_dir().join(layout_id.to_string())
    }

    /// Calculate the total size of the segment
    fn calculate_segment_size(&self) -> Result<u64> {
        let segment_dir = self.segment_dir();
        let mut total_size = 0u64;

        if segment_dir.exists() {
            for entry in std::fs::read_dir(&segment_dir)
                .map_err(|e| kylin_common::KylinError::Engine(format!("Failed to read directory: {}", e)))?
            {
                let entry = entry
                    .map_err(|e| kylin_common::KylinError::Engine(format!("Failed to read entry: {}", e)))?;
                let path = entry.path();

                if path.is_dir() {
                    for file in std::fs::read_dir(&path)
                        .map_err(|e| kylin_common::KylinError::Engine(format!("Failed to read directory: {}", e)))?
                    {
                        let file = file
                            .map_err(|e| kylin_common::KylinError::Engine(format!("Failed to read file: {}", e)))?;
                        total_size += file.metadata().map(|m| m.len()).unwrap_or(0);
                    }
                }
            }
        }

        Ok(total_size)
    }

    /// Update progress
    async fn update_progress(&mut self, progress: f32) -> Result<()> {
        self.progress = progress;
        tracing::info!("Build progress: {:.0}%", progress * 100.0);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kylin_common::types::{ModelType, PersistentEntity, SegmentStatus};
    use kylin_metadata::dataflow::{DataflowStatus, LayoutEntity};
    use kylin_metadata::sqlite_store::SqliteMetadataStore;
    use kylin_metadata::MetadataManager;

    #[tokio::test]
    async fn test_segment_build_job() {
        let store = SqliteMetadataStore::new_in_memory().await.unwrap();
        store.run_migrations().await.unwrap();

        let manager = MetadataManager::new(Arc::new(store));

        // Create project and model
        manager.create_project("test_project", Some("Test")).await.unwrap();
        let model = DataModel {
            entity: PersistentEntity::new(),
            name: "test_model".to_string(),
            root_fact_table: "DEFAULT.SALES".to_string(),
            model_type: ModelType::Batch,
            join_tables: vec![],
            all_columns: vec![],
            all_measures: vec![],
            filter_condition: None,
            partition_desc: None,
            computed_columns: vec![],
        };
        manager.store().save_model("test_project", &model).await.unwrap();

        // Create dataflow
        let dataflow = Dataflow {
            entity: PersistentEntity::new(),
            project: "test_project".to_string(),
            model_uuid: model.entity.uuid.clone(),
            model_name: model.name.clone(),
            status: DataflowStatus::Active,
            segments: vec![],
            layouts: vec![LayoutEntity {
                id: 1,
                dimensions: vec![],
                measures: vec![],
                shard_by_columns: vec![],
                is_table_index: true,
                storage_size: 0,
                row_count: 0,
            }],
        };
        manager.store().save_dataflow(&dataflow).await.unwrap();

        // Create segment
        let segment = Segment::new(&dataflow.entity.uuid, 1000000, 2000000);
        manager.store().save_segment(&segment).await.unwrap();

        // Create build job
        let data_dir = std::env::temp_dir().join("kylin_test");
        let mut job = SegmentBuildJob::new(
            model,
            dataflow,
            segment,
            manager.store().clone(),
            data_dir.clone(),
        );

        // Execute job
        job.execute().await.unwrap();

        // Verify segment is ready
        let segment = manager.store().load_segment(&job.segment.entity.uuid).await.unwrap();
        assert_eq!(segment.status, SegmentStatus::Ready);

        // Cleanup
        let _ = std::fs::remove_dir_all(&data_dir);
    }
}
