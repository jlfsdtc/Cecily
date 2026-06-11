use async_trait::async_trait;
use kylin_common::{KylinError, Result};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::Row;

use crate::dataflow::Dataflow;
use crate::model::DataModel;
use crate::project::Project;
use crate::segment::Segment;
use crate::store::MetadataStore;
use crate::table::TableDesc;

/// SQLite-backed metadata store
pub struct SqliteMetadataStore {
    pool: SqlitePool,
}

impl SqliteMetadataStore {
    /// Create a new SQLite metadata store
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .map_err(|e| KylinError::Metadata(format!("Failed to connect to SQLite: {}", e)))?;

        Ok(Self { pool })
    }

    /// Create a new SQLite metadata store with in-memory database (for testing)
    pub async fn new_in_memory() -> Result<Self> {
        Self::new("sqlite::memory:").await
    }

    /// Run migrations
    pub async fn run_migrations(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS kylin_project (
                uuid TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                default_database TEXT,
                active BOOLEAN NOT NULL DEFAULT 1,
                definition TEXT NOT NULL,
                last_modified BIGINT NOT NULL,
                version BIGINT NOT NULL DEFAULT 1,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| KylinError::Metadata(format!("Failed to create project table: {}", e)))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS kylin_model (
                uuid TEXT PRIMARY KEY,
                project TEXT NOT NULL,
                name TEXT NOT NULL,
                root_fact_table TEXT NOT NULL,
                model_type TEXT NOT NULL DEFAULT 'BATCH',
                definition TEXT NOT NULL,
                last_modified BIGINT NOT NULL,
                version BIGINT NOT NULL DEFAULT 1,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(project, name),
                FOREIGN KEY (project) REFERENCES kylin_project(name) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| KylinError::Metadata(format!("Failed to create model table: {}", e)))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS kylin_dataflow (
                uuid TEXT PRIMARY KEY,
                project TEXT NOT NULL,
                model_uuid TEXT NOT NULL,
                model_name TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'ACTIVE',
                definition TEXT NOT NULL,
                last_modified BIGINT NOT NULL,
                version BIGINT NOT NULL DEFAULT 1,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (project) REFERENCES kylin_project(name) ON DELETE CASCADE,
                FOREIGN KEY (model_uuid) REFERENCES kylin_model(uuid) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| KylinError::Metadata(format!("Failed to create dataflow table: {}", e)))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS kylin_segment (
                uuid TEXT PRIMARY KEY,
                dataflow_uuid TEXT NOT NULL,
                name TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'LOADING',
                time_range_start BIGINT NOT NULL,
                time_range_end BIGINT NOT NULL,
                source_count BIGINT DEFAULT 0,
                size_bytes BIGINT DEFAULT 0,
                definition TEXT NOT NULL,
                last_modified BIGINT NOT NULL,
                version BIGINT NOT NULL DEFAULT 1,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (dataflow_uuid) REFERENCES kylin_dataflow(uuid) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| KylinError::Metadata(format!("Failed to create segment table: {}", e)))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS kylin_table_desc (
                project TEXT NOT NULL,
                full_name TEXT NOT NULL,
                database_name TEXT NOT NULL,
                table_name TEXT NOT NULL,
                table_type TEXT NOT NULL DEFAULT 'TABLE',
                source_type TEXT NOT NULL DEFAULT 'HIVE',
                definition TEXT NOT NULL,
                last_modified BIGINT NOT NULL,
                PRIMARY KEY (project, full_name),
                FOREIGN KEY (project) REFERENCES kylin_project(name) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| KylinError::Metadata(format!("Failed to create table_desc table: {}", e)))?;

        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_model_project ON kylin_model(project)")
            .execute(&self.pool)
            .await
            .map_err(|e| KylinError::Metadata(format!("Failed to create index: {}", e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_dataflow_project ON kylin_dataflow(project)")
            .execute(&self.pool)
            .await
            .map_err(|e| KylinError::Metadata(format!("Failed to create index: {}", e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_dataflow_model ON kylin_dataflow(model_uuid)")
            .execute(&self.pool)
            .await
            .map_err(|e| KylinError::Metadata(format!("Failed to create index: {}", e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_segment_dataflow ON kylin_segment(dataflow_uuid)")
            .execute(&self.pool)
            .await
            .map_err(|e| KylinError::Metadata(format!("Failed to create index: {}", e)))?;

        Ok(())
    }

    /// Get the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[async_trait]
impl MetadataStore for SqliteMetadataStore {
    // ==================== Model Operations ====================

    async fn load_model(&self, project: &str, uuid: &str) -> Result<DataModel> {
        let row = sqlx::query(
            "SELECT definition FROM kylin_model WHERE project = ? AND uuid = ?"
        )
        .bind(project)
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| KylinError::Metadata(format!("Failed to load model: {}", e)))?;

        match row {
            Some(row) => {
                let definition: String = row.get("definition");
                let model: DataModel = serde_json::from_str(&definition)?;
                Ok(model)
            }
            None => Err(KylinError::NotFound(format!(
                "Model not found: project={}, uuid={}",
                project, uuid
            ))),
        }
    }

    async fn save_model(&self, project: &str, model: &DataModel) -> Result<()> {
        let definition = serde_json::to_string(model)?;
        let now = chrono::Utc::now().timestamp_millis();

        sqlx::query(
            r#"
            INSERT INTO kylin_model (uuid, project, name, root_fact_table, model_type, definition, last_modified, version)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(uuid) DO UPDATE SET
                name = excluded.name,
                root_fact_table = excluded.root_fact_table,
                model_type = excluded.model_type,
                definition = excluded.definition,
                last_modified = excluded.last_modified,
                version = excluded.version + 1
            "#
        )
        .bind(&model.entity.uuid)
        .bind(project)
        .bind(&model.name)
        .bind(&model.root_fact_table)
        .bind(model.model_type.to_string())
        .bind(&definition)
        .bind(now)
        .bind(model.entity.version)
        .execute(&self.pool)
        .await
        .map_err(|e| KylinError::Metadata(format!("Failed to save model: {}", e)))?;

        Ok(())
    }

    async fn list_models(&self, project: &str) -> Result<Vec<DataModel>> {
        let rows = sqlx::query(
            "SELECT definition FROM kylin_model WHERE project = ? ORDER BY name"
        )
        .bind(project)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| KylinError::Metadata(format!("Failed to list models: {}", e)))?;

        let mut models = Vec::new();
        for row in rows {
            let definition: String = row.get("definition");
            let model: DataModel = serde_json::from_str(&definition)?;
            models.push(model);
        }

        Ok(models)
    }

    async fn delete_model(&self, project: &str, uuid: &str) -> Result<()> {
        let result = sqlx::query(
            "DELETE FROM kylin_model WHERE project = ? AND uuid = ?"
        )
        .bind(project)
        .bind(uuid)
        .execute(&self.pool)
        .await
        .map_err(|e| KylinError::Metadata(format!("Failed to delete model: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(KylinError::NotFound(format!(
                "Model not found: project={}, uuid={}",
                project, uuid
            )));
        }

        Ok(())
    }

    // ==================== Dataflow Operations ====================

    async fn load_dataflow(&self, uuid: &str) -> Result<Dataflow> {
        let row = sqlx::query(
            "SELECT definition FROM kylin_dataflow WHERE uuid = ?"
        )
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| KylinError::Metadata(format!("Failed to load dataflow: {}", e)))?;

        match row {
            Some(row) => {
                let definition: String = row.get("definition");
                let dataflow: Dataflow = serde_json::from_str(&definition)?;
                Ok(dataflow)
            }
            None => Err(KylinError::NotFound(format!(
                "Dataflow not found: uuid={}",
                uuid
            ))),
        }
    }

    async fn save_dataflow(&self, dataflow: &Dataflow) -> Result<()> {
        let definition = serde_json::to_string(dataflow)?;
        let now = chrono::Utc::now().timestamp_millis();

        sqlx::query(
            r#"
            INSERT INTO kylin_dataflow (uuid, project, model_uuid, model_name, status, definition, last_modified, version)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(uuid) DO UPDATE SET
                status = excluded.status,
                definition = excluded.definition,
                last_modified = excluded.last_modified,
                version = excluded.version + 1
            "#
        )
        .bind(&dataflow.entity.uuid)
        .bind(&dataflow.project)
        .bind(&dataflow.model_uuid)
        .bind(&dataflow.model_name)
        .bind(dataflow.status.to_string())
        .bind(&definition)
        .bind(now)
        .bind(dataflow.entity.version)
        .execute(&self.pool)
        .await
        .map_err(|e| KylinError::Metadata(format!("Failed to save dataflow: {}", e)))?;

        Ok(())
    }

    async fn list_dataflows(&self, project: &str) -> Result<Vec<Dataflow>> {
        let rows = sqlx::query(
            "SELECT definition FROM kylin_dataflow WHERE project = ? ORDER BY model_name"
        )
        .bind(project)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| KylinError::Metadata(format!("Failed to list dataflows: {}", e)))?;

        let mut dataflows = Vec::new();
        for row in rows {
            let definition: String = row.get("definition");
            let dataflow: Dataflow = serde_json::from_str(&definition)?;
            dataflows.push(dataflow);
        }

        Ok(dataflows)
    }

    async fn delete_dataflow(&self, uuid: &str) -> Result<()> {
        let result = sqlx::query(
            "DELETE FROM kylin_dataflow WHERE uuid = ?"
        )
        .bind(uuid)
        .execute(&self.pool)
        .await
        .map_err(|e| KylinError::Metadata(format!("Failed to delete dataflow: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(KylinError::NotFound(format!(
                "Dataflow not found: uuid={}",
                uuid
            )));
        }

        Ok(())
    }

    // ==================== Segment Operations ====================

    async fn load_segment(&self, uuid: &str) -> Result<Segment> {
        let row = sqlx::query(
            "SELECT definition FROM kylin_segment WHERE uuid = ?"
        )
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| KylinError::Metadata(format!("Failed to load segment: {}", e)))?;

        match row {
            Some(row) => {
                let definition: String = row.get("definition");
                let segment: Segment = serde_json::from_str(&definition)?;
                Ok(segment)
            }
            None => Err(KylinError::NotFound(format!(
                "Segment not found: uuid={}",
                uuid
            ))),
        }
    }

    async fn save_segment(&self, segment: &Segment) -> Result<()> {
        let definition = serde_json::to_string(segment)?;
        let now = chrono::Utc::now().timestamp_millis();

        sqlx::query(
            r#"
            INSERT INTO kylin_segment (uuid, dataflow_uuid, name, status, time_range_start, time_range_end, source_count, size_bytes, definition, last_modified, version)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(uuid) DO UPDATE SET
                status = excluded.status,
                source_count = excluded.source_count,
                size_bytes = excluded.size_bytes,
                definition = excluded.definition,
                last_modified = excluded.last_modified,
                version = excluded.version + 1
            "#
        )
        .bind(&segment.entity.uuid)
        .bind(&segment.dataflow_uuid)
        .bind(&segment.name)
        .bind(segment.status.to_string())
        .bind(segment.time_range_start)
        .bind(segment.time_range_end)
        .bind(segment.source_count as i64)
        .bind(segment.size_bytes as i64)
        .bind(&definition)
        .bind(now)
        .bind(segment.entity.version)
        .execute(&self.pool)
        .await
        .map_err(|e| KylinError::Metadata(format!("Failed to save segment: {}", e)))?;

        Ok(())
    }

    async fn list_segments(&self, dataflow_uuid: &str) -> Result<Vec<Segment>> {
        let rows = sqlx::query(
            "SELECT definition FROM kylin_segment WHERE dataflow_uuid = ? ORDER BY time_range_start"
        )
        .bind(dataflow_uuid)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| KylinError::Metadata(format!("Failed to list segments: {}", e)))?;

        let mut segments = Vec::new();
        for row in rows {
            let definition: String = row.get("definition");
            let segment: Segment = serde_json::from_str(&definition)?;
            segments.push(segment);
        }

        Ok(segments)
    }

    async fn delete_segment(&self, uuid: &str) -> Result<()> {
        let result = sqlx::query(
            "DELETE FROM kylin_segment WHERE uuid = ?"
        )
        .bind(uuid)
        .execute(&self.pool)
        .await
        .map_err(|e| KylinError::Metadata(format!("Failed to delete segment: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(KylinError::NotFound(format!(
                "Segment not found: uuid={}",
                uuid
            )));
        }

        Ok(())
    }

    // ==================== Table Operations ====================

    async fn load_table(&self, project: &str, full_name: &str) -> Result<TableDesc> {
        let row = sqlx::query(
            "SELECT definition FROM kylin_table_desc WHERE project = ? AND full_name = ?"
        )
        .bind(project)
        .bind(full_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| KylinError::Metadata(format!("Failed to load table: {}", e)))?;

        match row {
            Some(row) => {
                let definition: String = row.get("definition");
                let table: TableDesc = serde_json::from_str(&definition)?;
                Ok(table)
            }
            None => Err(KylinError::NotFound(format!(
                "Table not found: project={}, full_name={}",
                project, full_name
            ))),
        }
    }

    async fn save_table(&self, project: &str, table: &TableDesc) -> Result<()> {
        let definition = serde_json::to_string(table)?;
        let now = chrono::Utc::now().timestamp_millis();

        sqlx::query(
            r#"
            INSERT INTO kylin_table_desc (project, full_name, database_name, table_name, table_type, source_type, definition, last_modified)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(project, full_name) DO UPDATE SET
                database_name = excluded.database_name,
                table_name = excluded.table_name,
                table_type = excluded.table_type,
                source_type = excluded.source_type,
                definition = excluded.definition,
                last_modified = excluded.last_modified
            "#
        )
        .bind(project)
        .bind(&table.full_name)
        .bind(&table.database)
        .bind(&table.name)
        .bind(table.table_type.to_string())
        .bind(&table.source_type)
        .bind(&definition)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| KylinError::Metadata(format!("Failed to save table: {}", e)))?;

        Ok(())
    }

    async fn list_tables(&self, project: &str) -> Result<Vec<TableDesc>> {
        let rows = sqlx::query(
            "SELECT definition FROM kylin_table_desc WHERE project = ? ORDER BY full_name"
        )
        .bind(project)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| KylinError::Metadata(format!("Failed to list tables: {}", e)))?;

        let mut tables = Vec::new();
        for row in rows {
            let definition: String = row.get("definition");
            let table: TableDesc = serde_json::from_str(&definition)?;
            tables.push(table);
        }

        Ok(tables)
    }

    // ==================== Project Operations ====================

    async fn load_project(&self, name: &str) -> Result<Project> {
        let row = sqlx::query(
            "SELECT definition FROM kylin_project WHERE name = ?"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| KylinError::Metadata(format!("Failed to load project: {}", e)))?;

        match row {
            Some(row) => {
                let definition: String = row.get("definition");
                let project: Project = serde_json::from_str(&definition)?;
                Ok(project)
            }
            None => Err(KylinError::NotFound(format!(
                "Project not found: name={}",
                name
            ))),
        }
    }

    async fn save_project(&self, project: &Project) -> Result<()> {
        let definition = serde_json::to_string(project)?;
        let now = chrono::Utc::now().timestamp_millis();

        sqlx::query(
            r#"
            INSERT INTO kylin_project (uuid, name, description, default_database, active, definition, last_modified, version)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(uuid) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                default_database = excluded.default_database,
                active = excluded.active,
                definition = excluded.definition,
                last_modified = excluded.last_modified,
                version = excluded.version + 1
            "#
        )
        .bind(&project.entity.uuid)
        .bind(&project.name)
        .bind(&project.description)
        .bind(&project.default_database)
        .bind(project.active)
        .bind(&definition)
        .bind(now)
        .bind(project.entity.version)
        .execute(&self.pool)
        .await
        .map_err(|e| KylinError::Metadata(format!("Failed to save project: {}", e)))?;

        Ok(())
    }

    async fn list_projects(&self) -> Result<Vec<Project>> {
        let rows = sqlx::query(
            "SELECT definition FROM kylin_project WHERE active = TRUE ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| KylinError::Metadata(format!("Failed to list projects: {}", e)))?;

        let mut projects = Vec::new();
        for row in rows {
            let definition: String = row.get("definition");
            let project: Project = serde_json::from_str(&definition)?;
            projects.push(project);
        }

        Ok(projects)
    }

    async fn delete_project(&self, name: &str) -> Result<()> {
        let result = sqlx::query(
            "DELETE FROM kylin_project WHERE name = ?"
        )
        .bind(name)
        .execute(&self.pool)
        .await
        .map_err(|e| KylinError::Metadata(format!("Failed to delete project: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(KylinError::NotFound(format!(
                "Project not found: name={}",
                name
            )));
        }

        Ok(())
    }
}
