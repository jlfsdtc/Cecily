use kylin_common::types::{ModelType, PersistentEntity, SegmentStatus};
use kylin_metadata::dataflow::{Dataflow, DataflowStatus, LayoutEntity};
use kylin_metadata::manager::MetadataManager;
use kylin_metadata::model::{ColumnDesc, DataModel, JoinTableDesc, MeasureDesc};
use kylin_metadata::project::Project;
use kylin_metadata::segment::Segment;
use kylin_metadata::sqlite_store::SqliteMetadataStore;
use kylin_metadata::table::{TableDesc, TableType};
use kylin_metadata::MetadataStore;
use std::sync::Arc;

async fn create_test_store() -> SqliteMetadataStore {
    let store = SqliteMetadataStore::new_in_memory().await.unwrap();
    store.run_migrations().await.unwrap();
    store
}

fn create_test_project(name: &str) -> Project {
    let mut project = Project::new(name);
    project.description = Some(format!("Test project {}", name));
    project.default_database = Some("DEFAULT".to_string());
    project
}

fn create_test_model(project: &str, name: &str) -> DataModel {
    DataModel {
        entity: PersistentEntity::new(),
        name: name.to_string(),
        root_fact_table: format!("{}.KYLIN_SALES", project),
        model_type: ModelType::Batch,
        join_tables: vec![],
        all_columns: vec![],
        all_measures: vec![],
        filter_condition: None,
        partition_desc: None,
        computed_columns: vec![],
    }
}

#[tokio::test]
async fn test_project_crud() {
    let store = create_test_store().await;

    // Create project
    let project = create_test_project("test_project");
    store.save_project(&project).await.unwrap();

    // Load project
    let loaded = store.load_project("test_project").await.unwrap();
    assert_eq!(loaded.name, "test_project");
    assert_eq!(loaded.description, Some("Test project test_project".to_string()));

    // List projects
    let projects = store.list_projects().await.unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].name, "test_project");

    // Update project
    let mut updated = loaded.clone();
    updated.description = Some("Updated description".to_string());
    store.save_project(&updated).await.unwrap();

    let loaded = store.load_project("test_project").await.unwrap();
    assert_eq!(loaded.description, Some("Updated description".to_string()));

    // Delete project
    store.delete_project("test_project").await.unwrap();
    let projects = store.list_projects().await.unwrap();
    assert_eq!(projects.len(), 0);
}

#[tokio::test]
async fn test_model_crud() {
    let store = create_test_store().await;

    // Create project first
    let project = create_test_project("test_project");
    store.save_project(&project).await.unwrap();

    // Create model
    let model = create_test_model("test_project", "test_model");
    store.save_model("test_project", &model).await.unwrap();

    // Load model
    let loaded = store.load_model("test_project", &model.entity.uuid).await.unwrap();
    assert_eq!(loaded.name, "test_model");
    assert_eq!(loaded.root_fact_table, "test_project.KYLIN_SALES");

    // List models
    let models = store.list_models("test_project").await.unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].name, "test_model");

    // Update model
    let mut updated = loaded.clone();
    updated.filter_condition = Some("amount > 100".to_string());
    store.save_model("test_project", &updated).await.unwrap();

    let loaded = store.load_model("test_project", &model.entity.uuid).await.unwrap();
    assert_eq!(loaded.filter_condition, Some("amount > 100".to_string()));

    // Delete model
    store.delete_model("test_project", &model.entity.uuid).await.unwrap();
    let models = store.list_models("test_project").await.unwrap();
    assert_eq!(models.len(), 0);
}

#[tokio::test]
async fn test_dataflow_crud() {
    let store = create_test_store().await;

    // Create project and model
    let project = create_test_project("test_project");
    store.save_project(&project).await.unwrap();

    let model = create_test_model("test_project", "test_model");
    store.save_model("test_project", &model).await.unwrap();

    // Create dataflow
    let dataflow = Dataflow {
        entity: PersistentEntity::new(),
        project: "test_project".to_string(),
        model_uuid: model.entity.uuid.clone(),
        model_name: "test_model".to_string(),
        status: DataflowStatus::Active,
        segments: vec![],
        layouts: vec![LayoutEntity {
            id: 1,
            dimensions: vec!["dim1".to_string(), "dim2".to_string()],
            measures: vec!["measure1".to_string()],
            shard_by_columns: vec![],
            is_table_index: false,
            storage_size: 0,
            row_count: 0,
        }],
    };
    store.save_dataflow(&dataflow).await.unwrap();

    // Load dataflow
    let loaded = store.load_dataflow(&dataflow.entity.uuid).await.unwrap();
    assert_eq!(loaded.project, "test_project");
    assert_eq!(loaded.model_name, "test_model");
    assert_eq!(loaded.layouts.len(), 1);

    // List dataflows
    let dataflows = store.list_dataflows("test_project").await.unwrap();
    assert_eq!(dataflows.len(), 1);

    // Delete dataflow
    store.delete_dataflow(&dataflow.entity.uuid).await.unwrap();
    let dataflows = store.list_dataflows("test_project").await.unwrap();
    assert_eq!(dataflows.len(), 0);
}

#[tokio::test]
async fn test_segment_crud() {
    let store = create_test_store().await;

    // Create project, model, and dataflow
    let project = create_test_project("test_project");
    store.save_project(&project).await.unwrap();

    let model = create_test_model("test_project", "test_model");
    store.save_model("test_project", &model).await.unwrap();

    let dataflow = Dataflow {
        entity: PersistentEntity::new(),
        project: "test_project".to_string(),
        model_uuid: model.entity.uuid.clone(),
        model_name: "test_model".to_string(),
        status: DataflowStatus::Active,
        segments: vec![],
        layouts: vec![],
    };
    store.save_dataflow(&dataflow).await.unwrap();

    // Create segment
    let segment = Segment::new(&dataflow.entity.uuid, 1000000, 2000000);
    store.save_segment(&segment).await.unwrap();

    // Load segment
    let loaded = store.load_segment(&segment.entity.uuid).await.unwrap();
    assert_eq!(loaded.dataflow_uuid, dataflow.entity.uuid);
    assert_eq!(loaded.time_range_start, 1000000);
    assert_eq!(loaded.time_range_end, 2000000);

    // List segments
    let segments = store.list_segments(&dataflow.entity.uuid).await.unwrap();
    assert_eq!(segments.len(), 1);

    // Update segment
    let mut updated = loaded.clone();
    updated.status = SegmentStatus::Ready;
    updated.source_count = 1000;
    store.save_segment(&updated).await.unwrap();

    let loaded = store.load_segment(&segment.entity.uuid).await.unwrap();
    assert_eq!(loaded.status, SegmentStatus::Ready);
    assert_eq!(loaded.source_count, 1000);

    // Delete segment
    store.delete_segment(&segment.entity.uuid).await.unwrap();
    let segments = store.list_segments(&dataflow.entity.uuid).await.unwrap();
    assert_eq!(segments.len(), 0);
}

#[tokio::test]
async fn test_table_crud() {
    let store = create_test_store().await;

    // Create project
    let project = create_test_project("test_project");
    store.save_project(&project).await.unwrap();

    // Create table
    let table = TableDesc {
        database: "DEFAULT".to_string(),
        name: "KYLIN_SALES".to_string(),
        full_name: "DEFAULT.KYLIN_SALES".to_string(),
        table_type: TableType::Table,
        columns: vec![],
        source_type: "HIVE".to_string(),
    };
    store.save_table("test_project", &table).await.unwrap();

    // Load table
    let loaded = store.load_table("test_project", "DEFAULT.KYLIN_SALES").await.unwrap();
    assert_eq!(loaded.name, "KYLIN_SALES");
    assert_eq!(loaded.database, "DEFAULT");

    // List tables
    let tables = store.list_tables("test_project").await.unwrap();
    assert_eq!(tables.len(), 1);
    assert_eq!(tables[0].name, "KYLIN_SALES");
}

#[tokio::test]
async fn test_not_found_errors() {
    let store = create_test_store().await;

    // Test project not found
    let result = store.load_project("nonexistent").await;
    assert!(result.is_err());

    // Test model not found
    let result = store.load_model("test_project", "nonexistent").await;
    assert!(result.is_err());

    // Test dataflow not found
    let result = store.load_dataflow("nonexistent").await;
    assert!(result.is_err());

    // Test segment not found
    let result = store.load_segment("nonexistent").await;
    assert!(result.is_err());

    // Test table not found
    let result = store.load_table("test_project", "nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_cascade_delete() {
    let store = create_test_store().await;

    // Create project
    let project = create_test_project("test_project");
    store.save_project(&project).await.unwrap();

    // Create model
    let model = create_test_model("test_project", "test_model");
    store.save_model("test_project", &model).await.unwrap();

    // Create dataflow
    let dataflow = Dataflow {
        entity: PersistentEntity::new(),
        project: "test_project".to_string(),
        model_uuid: model.entity.uuid.clone(),
        model_name: "test_model".to_string(),
        status: DataflowStatus::Active,
        segments: vec![],
        layouts: vec![],
    };
    store.save_dataflow(&dataflow).await.unwrap();

    // Create segment
    let segment = Segment::new(&dataflow.entity.uuid, 1000000, 2000000);
    store.save_segment(&segment).await.unwrap();

    // Delete project should cascade
    store.delete_project("test_project").await.unwrap();

    // Verify all related entities are deleted
    let projects = store.list_projects().await.unwrap();
    assert_eq!(projects.len(), 0);

    // Note: In a real database with foreign keys, these would be cascade deleted
    // For SQLite, we need to check manually
    let models = store.list_models("test_project").await.unwrap();
    assert_eq!(models.len(), 0);
}

#[tokio::test]
async fn test_multiple_entities() {
    let store = create_test_store().await;

    // Create project
    let project = create_test_project("test_project");
    store.save_project(&project).await.unwrap();

    // Create multiple models
    for i in 0..5 {
        let model = create_test_model("test_project", &format!("model_{}", i));
        store.save_model("test_project", &model).await.unwrap();
    }

    // List models
    let models = store.list_models("test_project").await.unwrap();
    assert_eq!(models.len(), 5);

    // Verify names
    let names: Vec<String> = models.iter().map(|m| m.name.clone()).collect();
    assert!(names.contains(&"model_0".to_string()));
    assert!(names.contains(&"model_4".to_string()));
}

#[tokio::test]
async fn test_metadata_manager() {
    let store = create_test_store().await;
    let manager = MetadataManager::new(Arc::new(store));

    // Create project
    let project = manager.create_project("test_project", Some("Test project")).await.unwrap();
    assert_eq!(project.name, "test_project");

    // Create model
    let model = manager.create_model("test_project", "test_model", "DEFAULT.KYLIN_SALES").await.unwrap();
    assert_eq!(model.name, "test_model");

    // Get model by name
    let found = manager.get_model_by_name("test_project", "test_model").await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "test_model");

    // Create dataflow
    let dataflow = manager.create_dataflow("test_project", &model).await.unwrap();
    assert_eq!(dataflow.model_name, "test_model");

    // Create segment
    let segment = manager.create_segment(&dataflow, 1000000, 2000000).await.unwrap();
    assert_eq!(segment.time_range_start, 1000000);

    // Get model context
    let context = manager.get_model_context("test_project", &model.entity.uuid).await.unwrap();
    assert!(context.is_some());
    let (loaded_model, loaded_dataflow, loaded_segments) = context.unwrap();
    assert_eq!(loaded_model.name, "test_model");
    assert_eq!(loaded_dataflow.model_name, "test_model");
    assert_eq!(loaded_segments.len(), 1);
}
