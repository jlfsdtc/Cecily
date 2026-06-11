use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Kylin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KylinConfig {
    /// Server host
    pub server_host: String,
    /// Server port
    pub server_port: u16,
    /// Database URL for metadata storage
    pub metadata_db_url: String,
    /// Data storage root path
    pub data_dir: PathBuf,
    /// Maximum concurrent build jobs
    pub max_concurrent_jobs: usize,
    /// Query result cache size
    pub query_cache_size: usize,
    /// Log level
    pub log_level: String,
}

impl Default for KylinConfig {
    fn default() -> Self {
        Self {
            server_host: "0.0.0.0".to_string(),
            server_port: 7070,
            metadata_db_url: "sqlite:kylin.db".to_string(),
            data_dir: PathBuf::from("./data"),
            max_concurrent_jobs: 4,
            query_cache_size: 1000,
            log_level: "info".to_string(),
        }
    }
}

impl KylinConfig {
    /// Load config from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(host) = std::env::var("KYLIN_SERVER_HOST") {
            config.server_host = host;
        }
        if let Ok(port) = std::env::var("KYLIN_SERVER_PORT") {
            if let Ok(port) = port.parse() {
                config.server_port = port;
            }
        }
        if let Ok(db_url) = std::env::var("KYLIN_METADATA_DB_URL") {
            config.metadata_db_url = db_url;
        }
        if let Ok(data_dir) = std::env::var("KYLIN_DATA_DIR") {
            config.data_dir = PathBuf::from(data_dir);
        }
        if let Ok(max_jobs) = std::env::var("KYLIN_MAX_CONCURRENT_JOBS") {
            if let Ok(max_jobs) = max_jobs.parse() {
                config.max_concurrent_jobs = max_jobs;
            }
        }
        if let Ok(log_level) = std::env::var("KYLIN_LOG_LEVEL") {
            config.log_level = log_level;
        }

        config
    }
}
