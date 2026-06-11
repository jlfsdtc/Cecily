use async_trait::async_trait;
use datafusion::catalog::{SchemaProvider, TableProvider};
use datafusion_common::Result as DFResult;
use kylin_common::Result;
use kylin_metadata::MetadataStore;
use kylin_storage::StorageProvider;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use crate::table::KylinModelTableProvider;

/// Kylin schema provider - maps models within a project to DataFusion schemas
pub struct KylinSchemaProvider {
    project: String,
    metadata_store: Arc<dyn MetadataStore>,
    tables: parking_lot::RwLock<HashMap<String, Arc<dyn TableProvider>>>,
}

impl std::fmt::Debug for KylinSchemaProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KylinSchemaProvider")
            .field("project", &self.project)
            .finish()
    }
}

impl KylinSchemaProvider {
    /// Create a new Kylin schema provider
    pub fn new(project: String, metadata_store: Arc<dyn MetadataStore>) -> Self {
        Self {
            project,
            metadata_store,
            tables: parking_lot::RwLock::new(HashMap::new()),
        }
    }

    /// Get the project name
    pub fn project(&self) -> &str {
        &self.project
    }

    /// Load tables from metadata store
    pub async fn load_tables(&self, storage: Arc<dyn StorageProvider>) -> Result<()> {
        let models = self.metadata_store.list_models(&self.project).await?;
        let dataflows = self.metadata_store.list_dataflows(&self.project).await?;

        let mut tables = self.tables.write();

        for model in models {
            let dataflow = dataflows
                .iter()
                .find(|d| d.model_uuid == model.entity.uuid)
                .cloned();

            let table = Arc::new(KylinModelTableProvider::new(
                model.clone(),
                dataflow,
                storage.clone(),
            ));
            tables.insert(model.name.clone(), table);
        }

        Ok(())
    }
}

#[async_trait]
impl SchemaProvider for KylinSchemaProvider {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn table_names(&self) -> Vec<String> {
        let tables = self.tables.read();
        tables.keys().cloned().collect()
    }

    async fn table(&self, name: &str) -> DFResult<Option<Arc<dyn TableProvider>>> {
        let tables = self.tables.read();
        Ok(tables.get(name).cloned())
    }

    fn register_table(
        &self,
        name: String,
        table: Arc<dyn TableProvider>,
    ) -> DFResult<Option<Arc<dyn TableProvider>>> {
        let mut tables = self.tables.write();
        Ok(tables.insert(name, table))
    }

    fn deregister_table(
        &self,
        name: &str,
    ) -> DFResult<Option<Arc<dyn TableProvider>>> {
        let mut tables = self.tables.write();
        Ok(tables.remove(name))
    }

    fn table_exist(&self, name: &str) -> bool {
        let tables = self.tables.read();
        tables.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kylin_common::types::{ModelType, PersistentEntity};
    use kylin_metadata::sqlite_store::SqliteMetadataStore;
    use kylin_metadata::MetadataManager;
    use kylin_metadata::model::DataModel;

    #[tokio::test]
    async fn test_schema_provider() {
        let store = SqliteMetadataStore::new_in_memory().await.unwrap();
        store.run_migrations().await.unwrap();

        let manager = MetadataManager::new(Arc::new(store));
        manager.create_project("test_project", Some("Test")).await.unwrap();

        // Create a model
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

        // Create mock storage
        struct MockStorage;
        #[async_trait]
        impl StorageProvider for MockStorage {
            async fn write_layout_data(&self, _: &kylin_storage::LayoutDescriptor, _: Vec<u8>) -> kylin_common::Result<()> {
                Ok(())
            }
            async fn read_layout_data(&self, _: &kylin_storage::LayoutDescriptor) -> kylin_common::Result<Vec<u8>> {
                Ok(vec![])
            }
            async fn delete_segment_data(&self, _: &str, _: &str) -> kylin_common::Result<()> {
                Ok(())
            }
            async fn list_segment_layouts(&self, _: &str, _: &str) -> kylin_common::Result<Vec<i64>> {
                Ok(vec![])
            }
        }

        let storage = Arc::new(MockStorage);
        let schema = KylinSchemaProvider::new("test_project".to_string(), manager.store().clone());
        schema.load_tables(storage).await.unwrap();

        let names = schema.table_names();
        assert_eq!(names.len(), 1);
        assert!(names.contains(&"test_model".to_string()));
        assert!(schema.table_exist("test_model"));
    }
}
