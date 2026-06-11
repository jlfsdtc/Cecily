use crate::context::{OlapQueryContext, QueryAnalyzer};
use crate::result::QueryResult;
use arrow::record_batch::RecordBatch;
use datafusion::catalog::CatalogProvider;
use datafusion::prelude::SessionContext;
use kylin_catalog::{KylinCatalogProvider, KylinSchemaProvider, register_kylin_udafs};
use kylin_common::Result;
use kylin_metadata::MetadataStore;
use kylin_storage::StorageProvider;
use std::sync::Arc;
use std::time::Instant;

/// Query executor - handles SQL query execution
pub struct QueryExecutor {
    metadata_store: Arc<dyn MetadataStore>,
    storage: Arc<dyn StorageProvider>,
}

impl QueryExecutor {
    /// Create a new query executor
    pub fn new(metadata_store: Arc<dyn MetadataStore>, storage: Arc<dyn StorageProvider>) -> Self {
        Self {
            metadata_store,
            storage,
        }
    }

    /// Execute a SQL query
    pub async fn execute(&self, sql: &str, project: &str) -> Result<QueryResult> {
        let start = Instant::now();
        tracing::info!("Executing query in project {}: {}", project, sql);

        // 1. Create SessionContext with Kylin catalog
        let ctx = self.create_session_context(project).await?;

        // 2. Analyze query
        let analyzer = QueryAnalyzer::new(project);
        let contexts = analyzer.analyze(sql);
        tracing::info!("Query analysis: {} contexts", contexts.len());

        // 3. Execute query via DataFusion
        let df = ctx
            .sql(sql)
            .await
            .map_err(|e| kylin_common::KylinError::Query(format!("Failed to parse SQL: {}", e)))?;

        // 4. Collect results
        let batches = df
            .collect()
            .await
            .map_err(|e| kylin_common::KylinError::Query(format!("Failed to execute query: {}", e)))?;

        // 5. Format results
        let execution_time = start.elapsed().as_millis() as u64;
        let result = QueryResult::from_record_batches(&batches, execution_time);

        tracing::info!(
            "Query executed in {}ms, returned {} rows",
            execution_time,
            result.row_count
        );

        Ok(result)
    }

    /// Create a DataFusion SessionContext with Kylin catalog
    async fn create_session_context(&self, project: &str) -> Result<SessionContext> {
        let ctx = SessionContext::new();

        // Register Kylin UDAFs
        register_kylin_udafs(&ctx)
            .map_err(|e| kylin_common::KylinError::Query(format!("Failed to register UDAFs: {}", e)))?;

        // Create and register catalog
        let catalog = KylinCatalogProvider::new(self.metadata_store.clone());
        catalog.load_schemas().await?;

        // Load tables for the project
        if let Some(schema) = catalog.schema(project) {
            if let Some(kylin_schema) = schema.as_any().downcast_ref::<KylinSchemaProvider>() {
                kylin_schema.load_tables(self.storage.clone()).await?;
            }
        }

        ctx.register_catalog(project, Arc::new(catalog));

        Ok(ctx)
    }

    /// Execute a query and return RecordBatches
    pub async fn execute_raw(&self, sql: &str, project: &str) -> Result<Vec<RecordBatch>> {
        let ctx = self.create_session_context(project).await?;

        let df = ctx
            .sql(sql)
            .await
            .map_err(|e| kylin_common::KylinError::Query(format!("Failed to parse SQL: {}", e)))?;

        let batches = df
            .collect()
            .await
            .map_err(|e| kylin_common::KylinError::Query(format!("Failed to execute query: {}", e)))?;

        Ok(batches)
    }

    /// Validate a SQL query without executing it
    pub async fn validate(&self, sql: &str, project: &str) -> Result<bool> {
        let ctx = self.create_session_context(project).await?;

        match ctx.sql(sql).await {
            Ok(_) => Ok(true),
            Err(e) => {
                tracing::warn!("SQL validation failed: {}", e);
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kylin_common::types::{ModelType, PersistentEntity};
    use kylin_metadata::model::DataModel;
    use kylin_metadata::sqlite_store::SqliteMetadataStore;
    use kylin_metadata::MetadataManager;

    async fn create_test_executor() -> (QueryExecutor, MetadataManager) {
        let store = SqliteMetadataStore::new_in_memory().await.unwrap();
        store.run_migrations().await.unwrap();

        let manager = MetadataManager::new(Arc::new(store));

        // Create mock storage
        struct MockStorage;
        #[async_trait::async_trait]
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
        let executor = QueryExecutor::new(manager.store().clone(), storage);

        (executor, manager)
    }

    #[tokio::test]
    async fn test_query_executor_creation() {
        let (executor, _) = create_test_executor().await;
        // Just verify creation works
        assert!(true);
    }

    #[tokio::test]
    async fn test_validate_query() {
        let (executor, manager) = create_test_executor().await;

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

        // Validate a simple query
        let result = executor.validate("SELECT 1", "test_project").await;
        assert!(result.is_ok());
    }
}
