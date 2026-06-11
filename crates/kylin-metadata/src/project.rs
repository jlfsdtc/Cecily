use kylin_common::types::PersistentEntity;
use serde::{Deserialize, Serialize};

/// A Kylin project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    #[serde(flatten)]
    pub entity: PersistentEntity,
    /// Project name
    pub name: String,
    /// Project description
    pub description: Option<String>,
    /// Default database
    pub default_database: Option<String>,
    /// Whether the project is active
    pub active: bool,
    /// Creation time
    pub create_time: i64,
}

impl Project {
    /// Create a new project
    pub fn new(name: &str) -> Self {
        Self {
            entity: PersistentEntity::new(),
            name: name.to_string(),
            description: None,
            default_database: None,
            active: true,
            create_time: chrono::Utc::now().timestamp_millis(),
        }
    }
}
