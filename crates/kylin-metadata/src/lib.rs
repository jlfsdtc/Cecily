pub mod model;
pub mod dataflow;
pub mod segment;
pub mod table;
pub mod project;
pub mod store;
pub mod sqlite_store;
pub mod postgres_store;
pub mod manager;

pub use model::DataModel;
pub use dataflow::Dataflow;
pub use segment::Segment;
pub use table::TableDesc;
pub use project::Project;
pub use store::MetadataStore;
pub use sqlite_store::SqliteMetadataStore;
pub use postgres_store::PostgresMetadataStore;
pub use manager::MetadataManager;

// Re-export common types for convenience
pub use kylin_common::types::{SegmentStatus, JobStatus, ModelType, DataStorageType};
