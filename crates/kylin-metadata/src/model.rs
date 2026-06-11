use kylin_common::types::{KylinDataType, ModelType, PersistentEntity};
use serde::{Deserialize, Serialize};

/// A Kylin data model definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataModel {
    #[serde(flatten)]
    pub entity: PersistentEntity,
    /// Model name
    pub name: String,
    /// Root fact table name (e.g., "DEFAULT.KYLIN_SALES")
    pub root_fact_table: String,
    /// Model type (Batch, Streaming, Hybrid)
    pub model_type: ModelType,
    /// Lookup table joins
    pub join_tables: Vec<JoinTableDesc>,
    /// All columns (dimensions + measures)
    pub all_columns: Vec<ColumnDesc>,
    /// All measures
    pub all_measures: Vec<MeasureDesc>,
    /// Filter condition
    pub filter_condition: Option<String>,
    /// Partition column
    pub partition_desc: Option<PartitionDesc>,
    /// Computed columns
    pub computed_columns: Vec<ComputedColumnDesc>,
}

/// Join table description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinTableDesc {
    /// Lookup table name
    pub table: String,
    /// Join type (INNER, LEFT, etc.)
    pub join_type: JoinType,
    /// Join conditions
    pub join_conditions: Vec<JoinCondition>,
}

/// Join type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

/// Join condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinCondition {
    /// Left column (fact table column)
    pub left_column: String,
    /// Right column (lookup table column)
    pub right_column: String,
}

/// Column description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDesc {
    /// Column UUID
    pub uuid: String,
    /// Column name
    pub name: String,
    /// Data type
    pub data_type: KylinDataType,
    /// Whether this is a dimension column
    pub is_dimension: bool,
    /// Whether this is a computed column
    pub is_computed: bool,
    /// Table name this column belongs to
    pub table_name: String,
    /// Column comment
    pub comment: Option<String>,
}

/// Measure description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasureDesc {
    /// Measure UUID
    pub uuid: String,
    /// Measure name
    pub name: String,
    /// Function description
    pub function: FunctionDesc,
    /// Column this measure operates on
    pub column: Option<String>,
    /// Measure comment
    pub comment: Option<String>,
}

/// Function description for measures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDesc {
    /// Function name (SUM, COUNT, COUNT_DISTINCT, TOP_N, etc.)
    pub name: String,
    /// Function parameters
    pub parameters: Vec<String>,
    /// Return type
    pub return_type: KylinDataType,
}

/// Partition description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionDesc {
    /// Partition column name
    pub partition_column: String,
    /// Partition column table
    pub partition_table: String,
    /// Partition date format
    pub date_format: Option<String>,
}

/// Computed column description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputedColumnDesc {
    /// Column UUID
    pub uuid: String,
    /// Column name
    pub name: String,
    /// Expression (e.g., "CASE WHEN amount > 100 THEN 'high' ELSE 'low' END")
    pub expression: String,
    /// Data type
    pub data_type: KylinDataType,
    /// Table this column belongs to
    pub table_name: String,
}
