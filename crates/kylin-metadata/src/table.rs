use kylin_common::types::KylinDataType;
use serde::{Deserialize, Serialize};

/// Table descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDesc {
    /// Database name
    pub database: String,
    /// Table name
    pub name: String,
    /// Full name (database.name)
    pub full_name: String,
    /// Table type
    pub table_type: TableType,
    /// Columns
    pub columns: Vec<TableColumn>,
    /// Source type (HIVE, JDBC, etc.)
    pub source_type: String,
}

/// Table type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TableType {
    Table,
    View,
    Lookup,
}

impl std::fmt::Display for TableType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TableType::Table => write!(f, "TABLE"),
            TableType::View => write!(f, "VIEW"),
            TableType::Lookup => write!(f, "LOOKUP"),
        }
    }
}

/// Table column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    /// Column name
    pub name: String,
    /// Data type
    pub data_type: KylinDataType,
    /// Whether nullable
    pub nullable: bool,
    /// Column comment
    pub comment: Option<String>,
    /// Column ID
    pub id: String,
}

/// Table extension descriptor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableExtDesc {
    /// Table name
    pub table_name: String,
    /// Last snapshot build time
    pub last_snapshot_build_time: Option<i64>,
    /// Last access time
    pub last_access_time: Option<i64>,
    /// Source location
    pub source_location: Option<String>,
}
