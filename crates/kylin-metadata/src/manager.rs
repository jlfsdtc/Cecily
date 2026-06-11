use crate::dataflow::Dataflow;
use crate::model::DataModel;
use crate::project::Project;
use crate::segment::Segment;
use crate::store::MetadataStore;
use crate::table::TableDesc;
use kylin_common::Result;
use std::sync::Arc;

/// Manager for metadata operations
/// Provides higher-level operations on top of MetadataStore
pub struct MetadataManager {
    store: Arc<dyn MetadataStore>,
}

impl MetadataManager {
    /// Create a new metadata manager
    pub fn new(store: Arc<dyn MetadataStore>) -> Self {
        Self { store }
    }

    /// Get the underlying store
    pub fn store(&self) -> &Arc<dyn MetadataStore> {
        &self.store
    }

    // ==================== Project Operations ====================

    /// Create a new project
    pub async fn create_project(&self, name: &str, description: Option<&str>) -> Result<Project> {
        let mut project = Project::new(name);
        project.description = description.map(|s| s.to_string());
        self.store.save_project(&project).await?;
        Ok(project)
    }

    /// Get or create a project
    pub async fn get_or_create_project(&self, name: &str) -> Result<Project> {
        match self.store.load_project(name).await {
            Ok(project) => Ok(project),
            Err(_) => self.create_project(name, None).await,
        }
    }

    // ==================== Model Operations ====================

    /// Create a new model
    pub async fn create_model(
        &self,
        project: &str,
        name: &str,
        root_fact_table: &str,
    ) -> Result<DataModel> {
        // Ensure project exists
        self.get_or_create_project(project).await?;

        let model = DataModel {
            entity: kylin_common::types::PersistentEntity::new(),
            name: name.to_string(),
            root_fact_table: root_fact_table.to_string(),
            model_type: kylin_common::types::ModelType::Batch,
            join_tables: vec![],
            all_columns: vec![],
            all_measures: vec![],
            filter_condition: None,
            partition_desc: None,
            computed_columns: vec![],
        };

        self.store.save_model(project, &model).await?;
        Ok(model)
    }

    /// Get a model by name
    pub async fn get_model_by_name(&self, project: &str, name: &str) -> Result<Option<DataModel>> {
        let models = self.store.list_models(project).await?;
        Ok(models.into_iter().find(|m| m.name == name))
    }

    /// Update model
    pub async fn update_model(&self, project: &str, model: &DataModel) -> Result<()> {
        self.store.save_model(project, model).await
    }

    // ==================== Dataflow Operations ====================

    /// Create a dataflow for a model
    pub async fn create_dataflow(&self, project: &str, model: &DataModel) -> Result<Dataflow> {
        let dataflow = Dataflow {
            entity: kylin_common::types::PersistentEntity::new(),
            project: project.to_string(),
            model_uuid: model.entity.uuid.clone(),
            model_name: model.name.clone(),
            status: crate::dataflow::DataflowStatus::Active,
            segments: vec![],
            layouts: vec![],
        };

        self.store.save_dataflow(&dataflow).await?;
        Ok(dataflow)
    }

    /// Get dataflows for a model
    pub async fn get_dataflows_for_model(
        &self,
        project: &str,
        model_uuid: &str,
    ) -> Result<Vec<Dataflow>> {
        let dataflows = self.store.list_dataflows(project).await?;
        Ok(dataflows
            .into_iter()
            .filter(|d| d.model_uuid == model_uuid)
            .collect())
    }

    /// Get or create a dataflow for a model
    pub async fn get_or_create_dataflow(
        &self,
        project: &str,
        model: &DataModel,
    ) -> Result<Dataflow> {
        let dataflows = self.get_dataflows_for_model(project, &model.entity.uuid).await?;
        match dataflows.into_iter().next() {
            Some(dataflow) => Ok(dataflow),
            None => self.create_dataflow(project, model).await,
        }
    }

    // ==================== Segment Operations ====================

    /// Create a segment for a dataflow
    pub async fn create_segment(
        &self,
        dataflow: &Dataflow,
        time_range_start: i64,
        time_range_end: i64,
    ) -> Result<Segment> {
        let segment = Segment::new(&dataflow.entity.uuid, time_range_start, time_range_end);
        self.store.save_segment(&segment).await?;
        Ok(segment)
    }

    /// Get segments for a dataflow
    pub async fn get_segments_for_dataflow(&self, dataflow_uuid: &str) -> Result<Vec<Segment>> {
        self.store.list_segments(dataflow_uuid).await
    }

    /// Get ready segments for a dataflow
    pub async fn get_ready_segments(&self, dataflow_uuid: &str) -> Result<Vec<Segment>> {
        let segments = self.store.list_segments(dataflow_uuid).await?;
        Ok(segments
            .into_iter()
            .filter(|s| s.status == kylin_common::types::SegmentStatus::Ready)
            .collect())
    }

    // ==================== Table Operations ====================

    /// Register a table
    pub async fn register_table(&self, project: &str, table: &TableDesc) -> Result<()> {
        self.store.save_table(project, table).await
    }

    /// Get all tables for a project
    pub async fn get_tables(&self, project: &str) -> Result<Vec<TableDesc>> {
        self.store.list_tables(project).await
    }

    // ==================== Query Helpers ====================

    /// Get model with its dataflow
    pub async fn get_model_with_dataflow(
        &self,
        project: &str,
        model_uuid: &str,
    ) -> Result<Option<(DataModel, Dataflow)>> {
        let model = self.store.load_model(project, model_uuid).await?;
        let dataflows = self.get_dataflows_for_model(project, model_uuid).await?;

        match dataflows.into_iter().next() {
            Some(dataflow) => Ok(Some((model, dataflow))),
            None => Ok(None),
        }
    }

    /// Get full model context (model + dataflow + segments)
    pub async fn get_model_context(
        &self,
        project: &str,
        model_uuid: &str,
    ) -> Result<Option<(DataModel, Dataflow, Vec<Segment>)>> {
        let model = self.store.load_model(project, model_uuid).await?;
        let dataflows = self.get_dataflows_for_model(project, model_uuid).await?;

        match dataflows.into_iter().next() {
            Some(dataflow) => {
                let segments = self.store.list_segments(&dataflow.entity.uuid).await?;
                Ok(Some((model, dataflow, segments)))
            }
            None => Ok(None),
        }
    }
}
