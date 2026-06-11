use arrow::datatypes::SchemaRef;
use async_trait::async_trait;
use datafusion::catalog::TableProvider;
use datafusion::datasource::TableType;
use datafusion::physical_plan::ExecutionPlan;
use datafusion::prelude::Expr;
use datafusion_common::Result as DFResult;
use datafusion_expr::TableProviderFilterPushDown;
use kylin_metadata::DataModel;
use kylin_metadata::dataflow::Dataflow;
use kylin_storage::StorageProvider;
use std::any::Any;
use std::sync::Arc;

use crate::conversion;
use crate::layout_chooser::{LayoutCandidate, LayoutChooser};

/// Kylin model table provider - exposes a Kylin model as a DataFusion table
pub struct KylinModelTableProvider {
    model: DataModel,
    dataflow: Option<Dataflow>,
    storage: Arc<dyn StorageProvider>,
    schema: SchemaRef,
}

impl std::fmt::Debug for KylinModelTableProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KylinModelTableProvider")
            .field("model", &self.model)
            .field("dataflow", &self.dataflow)
            .finish()
    }
}

impl KylinModelTableProvider {
    /// Create a new table provider for a model
    pub fn new(
        model: DataModel,
        dataflow: Option<Dataflow>,
        storage: Arc<dyn StorageProvider>,
    ) -> Self {
        let schema = conversion::model_to_arrow_schema(&model);
        Self {
            model,
            dataflow,
            storage,
            schema,
        }
    }

    /// Get the model
    pub fn model(&self) -> &DataModel {
        &self.model
    }

    /// Get the dataflow
    pub fn dataflow(&self) -> Option<&Dataflow> {
        self.dataflow.as_ref()
    }

    /// Find the best layout for the given query
    pub fn find_layout(&self, columns: &[String]) -> Option<LayoutCandidate> {
        match &self.dataflow {
            Some(dataflow) => LayoutChooser::choose_best_layout(&self.model, dataflow, columns),
            None => None,
        }
    }
}

#[async_trait]
impl TableProvider for KylinModelTableProvider {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }

    fn table_type(&self) -> TableType {
        TableType::Base
    }

    async fn scan(
        &self,
        _session: &dyn datafusion::catalog::Session,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        limit: Option<usize>,
    ) -> DFResult<Arc<dyn ExecutionPlan>> {
        // Get projected columns
        let projected_columns: Vec<String> = match projection {
            Some(indices) => indices
                .iter()
                .map(|i| self.schema.field(*i).name().clone())
                .collect(),
            None => self
                .schema
                .fields()
                .iter()
                .map(|f| f.name().clone())
                .collect(),
        };

        // Find the best layout
        let _layout = self.find_layout(&projected_columns);

        // Create a memory execution plan
        // In a real implementation, this would read from storage
        let plan = datafusion::physical_plan::empty::EmptyExec::new(self.schema.clone());

        Ok(Arc::new(plan))
    }

    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> DFResult<Vec<TableProviderFilterPushDown>> {
        // Support filter pushdown for all filters
        Ok(filters
            .iter()
            .map(|_| TableProviderFilterPushDown::Exact)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kylin_common::types::{KylinDataType, ModelType, PersistentEntity};
    use kylin_metadata::model::ColumnDesc;

    #[tokio::test]
    async fn test_table_provider_schema() {
        let model = DataModel {
            entity: PersistentEntity::new(),
            name: "test_model".to_string(),
            root_fact_table: "DEFAULT.SALES".to_string(),
            model_type: ModelType::Batch,
            join_tables: vec![],
            all_columns: vec![
                ColumnDesc {
                    uuid: "col1".to_string(),
                    name: "id".to_string(),
                    data_type: KylinDataType::BigInt,
                    is_dimension: true,
                    is_computed: false,
                    table_name: "SALES".to_string(),
                    comment: None,
                },
                ColumnDesc {
                    uuid: "col2".to_string(),
                    name: "amount".to_string(),
                    data_type: KylinDataType::Double,
                    is_dimension: false,
                    is_computed: false,
                    table_name: "SALES".to_string(),
                    comment: None,
                },
            ],
            all_measures: vec![],
            filter_condition: None,
            partition_desc: None,
            computed_columns: vec![],
        };

        // Create a mock storage provider
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
        let provider = KylinModelTableProvider::new(model, None, storage);

        let schema = provider.schema();
        assert_eq!(schema.fields().len(), 2);
        assert_eq!(schema.field(0).name(), "id");
        assert_eq!(schema.field(1).name(), "amount");
    }
}
