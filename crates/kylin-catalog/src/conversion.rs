use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use kylin_metadata::model::{ColumnDesc, DataModel};
use kylin_metadata::dataflow::LayoutEntity;
use kylin_common::types::KylinDataType;
use std::sync::Arc;

/// Convert KylinDataType to Arrow DataType
pub fn kylindata_to_arrow_type(dt: &KylinDataType) -> DataType {
    match dt {
        KylinDataType::TinyInt => DataType::Int8,
        KylinDataType::SmallInt => DataType::Int16,
        KylinDataType::Int => DataType::Int32,
        KylinDataType::BigInt => DataType::Int64,
        KylinDataType::Float => DataType::Float32,
        KylinDataType::Double => DataType::Float64,
        KylinDataType::Decimal { precision, scale } => {
            DataType::Decimal128(*precision, *scale as i8)
        }
        KylinDataType::Boolean => DataType::Boolean,
        KylinDataType::String => DataType::Utf8,
        KylinDataType::Varchar { .. } => DataType::Utf8,
        KylinDataType::Char { .. } => DataType::Utf8,
        KylinDataType::Date => DataType::Date32,
        KylinDataType::Timestamp => DataType::Timestamp(
            arrow::datatypes::TimeUnit::Millisecond,
            None,
        ),
        KylinDataType::Binary => DataType::Binary,
    }
}

/// Convert KylinDataType to nullable Arrow DataType
pub fn kylindata_to_arrow_field(name: &str, dt: &KylinDataType, nullable: bool) -> Field {
    let arrow_type = kylindata_to_arrow_type(dt);
    Field::new(name, arrow_type, nullable)
}

/// Convert a model's columns to Arrow Schema
pub fn model_to_arrow_schema(model: &DataModel) -> SchemaRef {
    let fields: Vec<Field> = model
        .all_columns
        .iter()
        .map(|col| {
            let nullable = true; // Most columns are nullable
            kylindata_to_arrow_field(&col.name, &col.data_type, nullable)
        })
        .collect();

    Arc::new(Schema::new(fields))
}

/// Convert a layout's dimensions and measures to Arrow Schema
pub fn layout_to_arrow_schema(
    model: &DataModel,
    layout: &LayoutEntity,
) -> SchemaRef {
    let mut fields = Vec::new();

    // Add dimension columns
    for dim_id in &layout.dimensions {
        if let Some(col) = model.all_columns.iter().find(|c| c.uuid == *dim_id) {
            fields.push(kylindata_to_arrow_field(&col.name, &col.data_type, true));
        }
    }

    // Add measure columns
    for measure_id in &layout.measures {
        if let Some(measure) = model.all_measures.iter().find(|m| m.uuid == *measure_id) {
            let nullable = true;
            fields.push(kylindata_to_arrow_field(
                &measure.name,
                &measure.function.return_type,
                nullable,
            ));
        }
    }

    Arc::new(Schema::new(fields))
}

/// Convert a column list to Arrow Schema
pub fn columns_to_arrow_schema(columns: &[ColumnDesc]) -> SchemaRef {
    let fields: Vec<Field> = columns
        .iter()
        .map(|col| kylindata_to_arrow_field(&col.name, &col.data_type, true))
        .collect();

    Arc::new(Schema::new(fields))
}

#[cfg(test)]
mod tests {
    use super::*;
    use kylin_common::types::PersistentEntity;

    #[test]
    fn test_kylindata_to_arrow_type() {
        assert_eq!(kylindata_to_arrow_type(&KylinDataType::Int), DataType::Int32);
        assert_eq!(kylindata_to_arrow_type(&KylinDataType::BigInt), DataType::Int64);
        assert_eq!(kylindata_to_arrow_type(&KylinDataType::Double), DataType::Float64);
        assert_eq!(kylindata_to_arrow_type(&KylinDataType::String), DataType::Utf8);
        assert_eq!(kylindata_to_arrow_type(&KylinDataType::Boolean), DataType::Boolean);
    }

    #[test]
    fn test_model_to_arrow_schema() {
        let model = DataModel {
            entity: PersistentEntity::new(),
            name: "test_model".to_string(),
            root_fact_table: "DEFAULT.SALES".to_string(),
            model_type: kylin_metadata::ModelType::Batch,
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

        let schema = model_to_arrow_schema(&model);
        assert_eq!(schema.fields().len(), 2);
        assert_eq!(schema.field(0).name(), "id");
        assert_eq!(schema.field(0).data_type(), &DataType::Int64);
        assert_eq!(schema.field(1).name(), "amount");
        assert_eq!(schema.field(1).data_type(), &DataType::Float64);
    }
}
