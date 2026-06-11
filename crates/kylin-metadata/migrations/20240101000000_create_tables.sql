-- Kylin Metadata Schema
-- Supports both SQLite and PostgreSQL

-- Projects table
CREATE TABLE IF NOT EXISTS kylin_project (
    uuid TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    default_database TEXT,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    definition JSONB NOT NULL,
    last_modified BIGINT NOT NULL,
    version BIGINT NOT NULL DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Models table
CREATE TABLE IF NOT EXISTS kylin_model (
    uuid TEXT PRIMARY KEY,
    project TEXT NOT NULL,
    name TEXT NOT NULL,
    root_fact_table TEXT NOT NULL,
    model_type TEXT NOT NULL DEFAULT 'BATCH',
    definition JSONB NOT NULL,
    last_modified BIGINT NOT NULL,
    version BIGINT NOT NULL DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(project, name),
    FOREIGN KEY (project) REFERENCES kylin_project(name) ON DELETE CASCADE
);

-- Dataflows table
CREATE TABLE IF NOT EXISTS kylin_dataflow (
    uuid TEXT PRIMARY KEY,
    project TEXT NOT NULL,
    model_uuid TEXT NOT NULL,
    model_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'ACTIVE',
    definition JSONB NOT NULL,
    last_modified BIGINT NOT NULL,
    version BIGINT NOT NULL DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project) REFERENCES kylin_project(name) ON DELETE CASCADE,
    FOREIGN KEY (model_uuid) REFERENCES kylin_model(uuid) ON DELETE CASCADE
);

-- Segments table
CREATE TABLE IF NOT EXISTS kylin_segment (
    uuid TEXT PRIMARY KEY,
    dataflow_uuid TEXT NOT NULL,
    name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'LOADING',
    time_range_start BIGINT NOT NULL,
    time_range_end BIGINT NOT NULL,
    source_count BIGINT DEFAULT 0,
    size_bytes BIGINT DEFAULT 0,
    definition JSONB NOT NULL,
    last_modified BIGINT NOT NULL,
    version BIGINT NOT NULL DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (dataflow_uuid) REFERENCES kylin_dataflow(uuid) ON DELETE CASCADE
);

-- Tables descriptor table
CREATE TABLE IF NOT EXISTS kylin_table_desc (
    project TEXT NOT NULL,
    full_name TEXT NOT NULL,
    database_name TEXT NOT NULL,
    table_name TEXT NOT NULL,
    table_type TEXT NOT NULL DEFAULT 'TABLE',
    source_type TEXT NOT NULL DEFAULT 'HIVE',
    definition JSONB NOT NULL,
    last_modified BIGINT NOT NULL,
    PRIMARY KEY (project, full_name),
    FOREIGN KEY (project) REFERENCES kylin_project(name) ON DELETE CASCADE
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_model_project ON kylin_model(project);
CREATE INDEX IF NOT EXISTS idx_model_name ON kylin_model(project, name);
CREATE INDEX IF NOT EXISTS idx_dataflow_project ON kylin_dataflow(project);
CREATE INDEX IF NOT EXISTS idx_dataflow_model ON kylin_dataflow(model_uuid);
CREATE INDEX IF NOT EXISTS idx_dataflow_status ON kylin_dataflow(status);
CREATE INDEX IF NOT EXISTS idx_segment_dataflow ON kylin_segment(dataflow_uuid);
CREATE INDEX IF NOT EXISTS idx_segment_status ON kylin_segment(status);
CREATE INDEX IF NOT EXISTS idx_segment_time_range ON kylin_segment(time_range_start, time_range_end);
