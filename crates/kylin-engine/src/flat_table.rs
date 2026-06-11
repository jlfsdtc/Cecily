use arrow::array::RecordBatch;
use arrow::datatypes::SchemaRef;
use datafusion::prelude::SessionContext;
use kylin_common::Result;
use kylin_metadata::DataModel;
use std::sync::Arc;

/// Flat table builder - joins fact table with lookup tables
pub struct FlatTableBuilder {
    model: DataModel,
    ctx: SessionContext,
}

impl FlatTableBuilder {
    /// Create a new flat table builder
    pub fn new(model: DataModel) -> Self {
        Self {
            model,
            ctx: SessionContext::new(),
        }
    }

    /// Build flat table by joining tables
    pub async fn build(&self) -> Result<RecordBatch> {
        tracing::info!("Building flat table for model: {}", self.model.name);

        // In a real implementation, this would:
        // 1. Load fact table from data source
        // 2. Join with each lookup table
        // 3. Apply model filter
        // 4. Add computed columns
        // 5. Return result as RecordBatch

        // For now, return an empty batch with the model's schema
        let schema = self.model_schema();
        let batch = RecordBatch::new_empty(schema);

        Ok(batch)
    }

    /// Get the schema for the model
    fn model_schema(&self) -> SchemaRef {
        use arrow::datatypes::{DataType, Field, Schema};

        let fields: Vec<Field> = self
            .model
            .all_columns
            .iter()
            .map(|col| {
                let data_type = match &col.data_type {
                    kylin_common::types::KylinDataType::TinyInt => DataType::Int8,
                    kylin_common::types::KylinDataType::SmallInt => DataType::Int16,
                    kylin_common::types::KylinDataType::Int => DataType::Int32,
                    kylin_common::types::KylinDataType::BigInt => DataType::Int64,
                    kylin_common::types::KylinDataType::Float => DataType::Float32,
                    kylin_common::types::KylinDataType::Double => DataType::Float64,
                    kylin_common::types::KylinDataType::Boolean => DataType::Boolean,
                    kylin_common::types::KylinDataType::String => DataType::Utf8,
                    _ => DataType::Utf8,
                };
                Field::new(&col.name, data_type, true)
            })
            .collect();

        Arc::new(Schema::new(fields))
    }

    /// Execute a SQL query to build the flat table
    pub async fn build_with_sql(&self, sql: &str) -> Result<RecordBatch> {
        tracing::info!("Building flat table with SQL: {}", sql);

        let df = self
            .ctx
            .sql(sql)
            .await
            .map_err(|e| kylin_common::KylinError::Engine(format!("Failed to parse SQL: {}", e)))?;

        let batches = df
            .collect()
            .await
            .map_err(|e| kylin_common::KylinError::Engine(format!("Failed to execute SQL: {}", e)))?;

        if batches.is_empty() {
            return Err(kylin_common::KylinError::Engine(
                "No data returned from SQL".to_string(),
            ));
        }

        // Combine all batches into one
        let batch = arrow::compute::concat_batches(&batches[0].schema(), &batches)
            .map_err(|e| kylin_common::KylinError::Engine(format!("Failed to concat batches: {}", e)))?;

        Ok(batch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kylin_common::types::{KylinDataType, ModelType, PersistentEntity};
    use kylin_metadata::model::ColumnDesc;

    #[tokio::test]
    async fn test_flat_table_builder() {
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

        let builder = FlatTableBuilder::new(model);
        let batch = builder.build().await.unwrap();

        assert_eq!(batch.num_columns(), 2);
        assert_eq!(batch.schema().field(0).name(), "id");
        assert_eq!(batch.schema().field(1).name(), "amount");
    }
}
