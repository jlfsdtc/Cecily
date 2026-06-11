use arrow::array::RecordBatch;
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use datafusion::prelude::SessionContext;
use kylin_common::Result;
use kylin_metadata::DataModel;
use kylin_metadata::dataflow::LayoutEntity;
use std::sync::Arc;

/// Layout builder - computes a single layout
pub struct LayoutBuilder {
    model: DataModel,
    layout: LayoutEntity,
    ctx: SessionContext,
}

impl LayoutBuilder {
    /// Create a new layout builder
    pub fn new(model: DataModel, layout: LayoutEntity) -> Self {
        Self {
            model,
            layout,
            ctx: SessionContext::new(),
        }
    }

    /// Build layout from flat table data
    pub async fn build(&self, flat_table: &RecordBatch) -> Result<RecordBatch> {
        tracing::info!("Building layout: {}", self.layout.id);

        // Register the flat table as a view
        self.ctx
            .register_batch("flat_table", flat_table.clone())
            .map_err(|e| kylin_common::KylinError::Engine(format!("Failed to register table: {}", e)))?;

        // Build the aggregation query
        let sql = self.build_aggregation_sql();

        tracing::info!("Executing layout SQL: {}", sql);

        let df = self
            .ctx
            .sql(&sql)
            .await
            .map_err(|e| kylin_common::KylinError::Engine(format!("Failed to parse SQL: {}", e)))?;

        let batches = df
            .collect()
            .await
            .map_err(|e| kylin_common::KylinError::Engine(format!("Failed to execute SQL: {}", e)))?;

        if batches.is_empty() {
            return Ok(RecordBatch::new_empty(self.layout_schema()));
        }

        // Combine all batches into one
        let batch = arrow::compute::concat_batches(&batches[0].schema(), &batches)
            .map_err(|e| kylin_common::KylinError::Engine(format!("Failed to concat batches: {}", e)))?;

        tracing::info!(
            "Layout {} built with {} rows",
            self.layout.id,
            batch.num_rows()
        );

        Ok(batch)
    }

    /// Build the aggregation SQL for this layout
    fn build_aggregation_sql(&self) -> String {
        let mut dims = Vec::new();
        let mut aggs = Vec::new();

        // Collect dimension columns
        for dim_id in &self.layout.dimensions {
            if let Some(col) = self.model.all_columns.iter().find(|c| c.uuid == *dim_id) {
                dims.push(col.name.clone());
            }
        }

        // Collect measure columns
        for measure_id in &self.layout.measures {
            if let Some(measure) = self.model.all_measures.iter().find(|m| m.uuid == *measure_id) {
                let agg = match measure.function.name.as_str() {
                    "SUM" => format!("SUM({}) AS {}", measure.column.as_deref().unwrap_or(&measure.name), measure.name),
                    "COUNT" => format!("COUNT({}) AS {}", measure.column.as_deref().unwrap_or("*"), measure.name),
                    "COUNT_DISTINCT" => format!("COUNT(DISTINCT {}) AS {}", measure.column.as_deref().unwrap_or(&measure.name), measure.name),
                    "MIN" => format!("MIN({}) AS {}", measure.column.as_deref().unwrap_or(&measure.name), measure.name),
                    "MAX" => format!("MAX({}) AS {}", measure.column.as_deref().unwrap_or(&measure.name), measure.name),
                    "AVG" => format!("AVG({}) AS {}", measure.column.as_deref().unwrap_or(&measure.name), measure.name),
                    _ => format!("SUM({}) AS {}", measure.column.as_deref().unwrap_or(&measure.name), measure.name),
                };
                aggs.push(agg);
            }
        }

        // Build SQL
        if dims.is_empty() && aggs.is_empty() {
            return "SELECT * FROM flat_table".to_string();
        }

        let select = if dims.is_empty() {
            aggs.join(", ")
        } else if aggs.is_empty() {
            dims.join(", ")
        } else {
            format!("{}, {}", dims.join(", "), aggs.join(", "))
        };

        if dims.is_empty() {
            format!("SELECT {} FROM flat_table", select)
        } else {
            format!("SELECT {} FROM flat_table GROUP BY {}", select, dims.join(", "))
        }
    }

    /// Get the schema for this layout
    fn layout_schema(&self) -> SchemaRef {
        let mut fields = Vec::new();

        // Add dimension columns
        for dim_id in &self.layout.dimensions {
            if let Some(col) = self.model.all_columns.iter().find(|c| c.uuid == *dim_id) {
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
                fields.push(Field::new(&col.name, data_type, true));
            }
        }

        // Add measure columns
        for measure_id in &self.layout.measures {
            if let Some(measure) = self.model.all_measures.iter().find(|m| m.uuid == *measure_id) {
                let data_type = match &measure.function.return_type {
                    kylin_common::types::KylinDataType::TinyInt => DataType::Int8,
                    kylin_common::types::KylinDataType::SmallInt => DataType::Int16,
                    kylin_common::types::KylinDataType::Int => DataType::Int32,
                    kylin_common::types::KylinDataType::BigInt => DataType::Int64,
                    kylin_common::types::KylinDataType::Float => DataType::Float32,
                    kylin_common::types::KylinDataType::Double => DataType::Float64,
                    _ => DataType::Float64,
                };
                fields.push(Field::new(&measure.name, data_type, true));
            }
        }

        Arc::new(Schema::new(fields))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Float64Array, Int64Array, StringArray};
    use kylin_common::types::{KylinDataType, ModelType, PersistentEntity};
    use kylin_metadata::model::{ColumnDesc, FunctionDesc, MeasureDesc};

    #[tokio::test]
    async fn test_layout_builder() {
        let model = DataModel {
            entity: PersistentEntity::new(),
            name: "test_model".to_string(),
            root_fact_table: "DEFAULT.SALES".to_string(),
            model_type: ModelType::Batch,
            join_tables: vec![],
            all_columns: vec![
                ColumnDesc {
                    uuid: "col1".to_string(),
                    name: "category".to_string(),
                    data_type: KylinDataType::String,
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
            all_measures: vec![MeasureDesc {
                uuid: "m1".to_string(),
                name: "total_amount".to_string(),
                function: FunctionDesc {
                    name: "SUM".to_string(),
                    parameters: vec![],
                    return_type: KylinDataType::Double,
                },
                column: Some("amount".to_string()),
                comment: None,
            }],
            filter_condition: None,
            partition_desc: None,
            computed_columns: vec![],
        };

        let layout = LayoutEntity {
            id: 1,
            dimensions: vec!["col1".to_string()],
            measures: vec!["m1".to_string()],
            shard_by_columns: vec![],
            is_table_index: false,
            storage_size: 0,
            row_count: 0,
        };

        let builder = LayoutBuilder::new(model, layout);

        // Create test data
        let schema = Arc::new(Schema::new(vec![
            Field::new("category", DataType::Utf8, true),
            Field::new("amount", DataType::Float64, true),
        ]));

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(vec!["A", "B", "A", "B"])),
                Arc::new(Float64Array::from(vec![10.0, 20.0, 30.0, 40.0])),
            ],
        )
        .unwrap();

        let result = builder.build(&batch).await.unwrap();

        assert_eq!(result.num_columns(), 2); // category + total_amount
        assert_eq!(result.num_rows(), 2); // 2 groups (A, B)
    }
}
