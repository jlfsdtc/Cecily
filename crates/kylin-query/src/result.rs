use arrow::array::{Array, AsArray};
use arrow::datatypes::DataType;
use arrow::record_batch::RecordBatch;
use serde::{Deserialize, Serialize};

/// Query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub column_names: Vec<String>,
    pub column_types: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
    pub execution_time_ms: u64,
    pub scan_rows: u64,
    pub scan_bytes: u64,
}

impl QueryResult {
    pub fn empty() -> Self {
        Self {
            column_names: Vec::new(),
            column_types: Vec::new(),
            rows: Vec::new(),
            row_count: 0,
            execution_time_ms: 0,
            scan_rows: 0,
            scan_bytes: 0,
        }
    }

    /// Create QueryResult from Arrow RecordBatches
    pub fn from_record_batches(
        batches: &[RecordBatch],
        execution_time_ms: u64,
    ) -> Self {
        if batches.is_empty() {
            return Self::empty();
        }

        let schema = batches[0].schema();
        let column_names: Vec<String> = schema
            .fields()
            .iter()
            .map(|f| f.name().clone())
            .collect();
        let column_types: Vec<String> = schema
            .fields()
            .iter()
            .map(|f| format!("{:?}", f.data_type()))
            .collect();

        let mut rows = Vec::new();
        let mut total_rows = 0;

        for batch in batches {
            let num_rows = batch.num_rows();
            total_rows += num_rows;

            for row_idx in 0..num_rows {
                let mut row = Vec::new();
                for col_idx in 0..batch.num_columns() {
                    let column = batch.column(col_idx);
                    let value = extract_value(column, row_idx);
                    row.push(value);
                }
                rows.push(row);
            }
        }

        Self {
            column_names,
            column_types,
            rows,
            row_count: total_rows,
            execution_time_ms,
            scan_rows: total_rows as u64,
            scan_bytes: 0,
        }
    }

    /// Convert to JSON response format
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "columnMetas": self.column_names.iter().zip(self.column_types.iter())
                .map(|(name, typ)| serde_json::json!({
                    "name": name,
                    "dataType": typ
                }))
                .collect::<Vec<_>>(),
            "results": self.rows,
            "duration": self.execution_time_ms,
            "totalScanRows": self.scan_rows,
            "totalScanBytes": self.scan_bytes,
            "totalRemainingRows": self.row_count,
        })
    }
}

/// Extract a value from an Arrow array at a given index
fn extract_value(array: &dyn Array, index: usize) -> serde_json::Value {
    if array.is_null(index) {
        return serde_json::Value::Null;
    }

    match array.data_type() {
        DataType::Boolean => {
            let arr = array.as_boolean();
            serde_json::Value::Bool(arr.value(index))
        }
        DataType::Int8 => {
            let arr = array.as_primitive::<arrow::datatypes::Int8Type>();
            serde_json::Value::Number(arr.value(index).into())
        }
        DataType::Int16 => {
            let arr = array.as_primitive::<arrow::datatypes::Int16Type>();
            serde_json::Value::Number(arr.value(index).into())
        }
        DataType::Int32 => {
            let arr = array.as_primitive::<arrow::datatypes::Int32Type>();
            serde_json::Value::Number(arr.value(index).into())
        }
        DataType::Int64 => {
            let arr = array.as_primitive::<arrow::datatypes::Int64Type>();
            serde_json::Value::Number(arr.value(index).into())
        }
        DataType::UInt8 => {
            let arr = array.as_primitive::<arrow::datatypes::UInt8Type>();
            serde_json::Value::Number(arr.value(index).into())
        }
        DataType::UInt16 => {
            let arr = array.as_primitive::<arrow::datatypes::UInt16Type>();
            serde_json::Value::Number(arr.value(index).into())
        }
        DataType::UInt32 => {
            let arr = array.as_primitive::<arrow::datatypes::UInt32Type>();
            serde_json::Value::Number(arr.value(index).into())
        }
        DataType::UInt64 => {
            let arr = array.as_primitive::<arrow::datatypes::UInt64Type>();
            serde_json::Value::Number(arr.value(index).into())
        }
        DataType::Float32 => {
            let arr = array.as_primitive::<arrow::datatypes::Float32Type>();
            serde_json::json!(arr.value(index))
        }
        DataType::Float64 => {
            let arr = array.as_primitive::<arrow::datatypes::Float64Type>();
            serde_json::json!(arr.value(index))
        }
        DataType::Utf8 => {
            let arr = array.as_string::<i32>();
            serde_json::Value::String(arr.value(index).to_string())
        }
        DataType::LargeUtf8 => {
            let arr = array.as_string::<i64>();
            serde_json::Value::String(arr.value(index).to_string())
        }
        DataType::Date32 => {
            let arr = array.as_primitive::<arrow::datatypes::Date32Type>();
            serde_json::Value::Number(arr.value(index).into())
        }
        DataType::Date64 => {
            let arr = array.as_primitive::<arrow::datatypes::Date64Type>();
            serde_json::Value::Number(arr.value(index).into())
        }
        _ => {
            // For unsupported types, convert to string
            serde_json::Value::String(format!("{:?}", array))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Int64Array, StringArray, Float64Array};
    use arrow::datatypes::{Field, Schema};
    use std::sync::Arc;

    #[test]
    fn test_from_record_batches() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("name", DataType::Utf8, true),
            Field::new("amount", DataType::Float64, true),
        ]));

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int64Array::from(vec![1, 2, 3])),
                Arc::new(StringArray::from(vec!["a", "b", "c"])),
                Arc::new(Float64Array::from(vec![1.1, 2.2, 3.3])),
            ],
        )
        .unwrap();

        let result = QueryResult::from_record_batches(&[batch], 100);

        assert_eq!(result.column_names, vec!["id", "name", "amount"]);
        assert_eq!(result.row_count, 3);
        assert_eq!(result.rows.len(), 3);
        assert_eq!(result.execution_time_ms, 100);
    }

    #[test]
    fn test_empty_result() {
        let result = QueryResult::empty();
        assert_eq!(result.row_count, 0);
        assert!(result.rows.is_empty());
    }

    #[test]
    fn test_to_json() {
        let result = QueryResult {
            column_names: vec!["id".to_string()],
            column_types: vec!["Int64".to_string()],
            rows: vec![vec![serde_json::json!(1)]],
            row_count: 1,
            execution_time_ms: 50,
            scan_rows: 1,
            scan_bytes: 100,
        };

        let json = result.to_json();
        assert!(json["columnMetas"].is_array());
        assert!(json["results"].is_array());
        assert_eq!(json["duration"], 50);
    }
}
