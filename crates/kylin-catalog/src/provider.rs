use datafusion::catalog::{CatalogProvider, SchemaProvider};
use datafusion_common::Result as DFResult;
use kylin_common::Result;
use kylin_metadata::MetadataStore;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use crate::schema::KylinSchemaProvider;

/// Kylin catalog provider - maps projects to DataFusion catalogs
pub struct KylinCatalogProvider {
    metadata_store: Arc<dyn MetadataStore>,
    schemas: parking_lot::RwLock<HashMap<String, Arc<dyn SchemaProvider>>>,
}

impl std::fmt::Debug for KylinCatalogProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KylinCatalogProvider")
            .field("schemas", &self.schemas.read().keys().collect::<Vec<_>>())
            .finish()
    }
}

impl KylinCatalogProvider {
    /// Create a new Kylin catalog provider
    pub fn new(metadata_store: Arc<dyn MetadataStore>) -> Self {
        Self {
            metadata_store,
            schemas: parking_lot::RwLock::new(HashMap::new()),
        }
    }

    /// Get the metadata store
    pub fn metadata_store(&self) -> &Arc<dyn MetadataStore> {
        &self.metadata_store
    }

    /// Load schemas from metadata store
    pub async fn load_schemas(&self) -> Result<()> {
        let projects = self.metadata_store.list_projects().await?;
        let mut schemas = self.schemas.write();

        for project in projects {
            let schema = Arc::new(KylinSchemaProvider::new(
                project.name.clone(),
                self.metadata_store.clone(),
            ));
            schemas.insert(project.name, schema);
        }

        Ok(())
    }
}

impl CatalogProvider for KylinCatalogProvider {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn schema_names(&self) -> Vec<String> {
        let schemas = self.schemas.read();
        schemas.keys().cloned().collect()
    }

    fn schema(&self, name: &str) -> Option<Arc<dyn SchemaProvider>> {
        let schemas = self.schemas.read();
        schemas.get(name).cloned()
    }

    fn register_schema(
        &self,
        name: &str,
        schema: Arc<dyn SchemaProvider>,
    ) -> DFResult<Option<Arc<dyn SchemaProvider>>> {
        let mut schemas = self.schemas.write();
        Ok(schemas.insert(name.to_string(), schema))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kylin_metadata::sqlite_store::SqliteMetadataStore;
    use kylin_metadata::MetadataManager;

    #[tokio::test]
    async fn test_catalog_provider() {
        let store = SqliteMetadataStore::new_in_memory().await.unwrap();
        store.run_migrations().await.unwrap();

        let manager = MetadataManager::new(Arc::new(store));
        manager.create_project("test_project", Some("Test")).await.unwrap();

        let store = manager.store().clone();
        let catalog = KylinCatalogProvider::new(store);
        catalog.load_schemas().await.unwrap();

        let names = catalog.schema_names();
        assert_eq!(names.len(), 1);
        assert!(names.contains(&"test_project".to_string()));

        let schema = catalog.schema("test_project");
        assert!(schema.is_some());
    }
}
