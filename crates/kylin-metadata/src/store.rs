use crate::dataflow::Dataflow;
use crate::model::DataModel;
use crate::project::Project;
use crate::segment::Segment;
use crate::table::TableDesc;
use async_trait::async_trait;
use kylin_common::Result;

/// Metadata store trait for persistence
#[async_trait]
pub trait MetadataStore: Send + Sync {
    // Model operations
    async fn load_model(&self, project: &str, uuid: &str) -> Result<DataModel>;
    async fn save_model(&self, project: &str, model: &DataModel) -> Result<()>;
    async fn list_models(&self, project: &str) -> Result<Vec<DataModel>>;
    async fn delete_model(&self, project: &str, uuid: &str) -> Result<()>;

    // Dataflow operations
    async fn load_dataflow(&self, uuid: &str) -> Result<Dataflow>;
    async fn save_dataflow(&self, dataflow: &Dataflow) -> Result<()>;
    async fn list_dataflows(&self, project: &str) -> Result<Vec<Dataflow>>;
    async fn delete_dataflow(&self, uuid: &str) -> Result<()>;

    // Segment operations
    async fn load_segment(&self, uuid: &str) -> Result<Segment>;
    async fn save_segment(&self, segment: &Segment) -> Result<()>;
    async fn list_segments(&self, dataflow_uuid: &str) -> Result<Vec<Segment>>;
    async fn delete_segment(&self, uuid: &str) -> Result<()>;

    // Table operations
    async fn load_table(&self, project: &str, full_name: &str) -> Result<TableDesc>;
    async fn save_table(&self, project: &str, table: &TableDesc) -> Result<()>;
    async fn list_tables(&self, project: &str) -> Result<Vec<TableDesc>>;

    // Project operations
    async fn load_project(&self, name: &str) -> Result<Project>;
    async fn save_project(&self, project: &Project) -> Result<()>;
    async fn list_projects(&self) -> Result<Vec<Project>>;
    async fn delete_project(&self, name: &str) -> Result<()>;
}
