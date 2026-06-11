use thiserror::Error;

pub type Result<T> = std::result::Result<T, KylinError>;

#[derive(Error, Debug)]
pub enum KylinError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Metadata error: {0}")]
    Metadata(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Engine error: {0}")]
    Engine(String),

    #[error("Job error: {0}")]
    Job(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
