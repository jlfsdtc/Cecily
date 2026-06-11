# Kylin-Rust Architecture

## Overview

Kylin-Rust is a high-performance OLAP analytics engine built with Rust, replacing the original Java/Spark-based Apache Kylin. It uses Apache DataFusion for query execution and build pipeline, providing significant performance improvements and reduced resource consumption.

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Vue.js Frontend                        │
│                     (kystudio/dist)                          │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Axum HTTP Server                          │
│                    (kylin-server)                            │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │  Models  │  │  Query   │  │   Jobs   │  │ Projects │   │
│  │ Endpoint │  │ Endpoint │  │ Endpoint │  │ Endpoint │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   Metadata   │    │    Query     │    │    Build     │
│    Store     │    │    Engine    │    │    Engine    │
│  (kylin-     │    │  (kylin-     │    │  (kylin-     │
│  metadata)   │    │  query)      │    │  engine)     │
└──────────────┘    └──────────────┘    └──────────────┘
        │                   │                   │
        ▼                   ▼                   ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   SQLite/    │    │  DataFusion  │    │  DataFusion  │
│  PostgreSQL  │    │   Session    │    │   + Parquet  │
└──────────────┘    └──────────────┘    └──────────────┘
                              │
                              ▼
                    ┌──────────────┐
                    │   Storage    │
                    │  (Local/S3)  │
                    └──────────────┘
```

## Component Overview

### 1. kylin-common
Shared types, configuration, and error handling used across all crates.

**Key Types:**
- `KylinConfig` - Server configuration
- `PersistentEntity` - Base entity with UUID and timestamps
- `KylinError` - Error types

### 2. kylin-metadata
Metadata storage and management for models, dataflows, segments, and projects.

**Key Components:**
- `MetadataStore` trait - CRUD operations interface
- `SqliteMetadataStore` - SQLite implementation
- `PostgresMetadataStore` - PostgreSQL implementation
- `MetadataManager` - High-level operations

**Data Model:**
```
Project
  └── Model
        └── Dataflow
              └── Segment
                    └── Layout
```

### 3. kylin-storage
Storage abstraction for reading and writing data.

**Key Components:**
- `StorageProvider` trait - Storage interface
- `LocalStorageProvider` - Local filesystem
- `LayoutDescriptor` - Layout file metadata

### 4. kylin-catalog
DataFusion integration for catalog and schema management.

**Key Components:**
- `KylinCatalogProvider` - Maps projects to DataFusion catalogs
- `KylinSchemaProvider` - Maps models to DataFusion schemas
- `KylinModelTableProvider` - Exposes models as queryable tables
- `LayoutChooser` - Selects optimal layout for queries

### 5. kylin-query
Query execution engine using DataFusion.

**Key Components:**
- `QueryExecutor` - Executes SQL queries
- `OlapQueryContext` - Query analysis context
- `QueryAnalyzer` - Extracts tables and columns from SQL
- `QueryResult` - Query result formatting

**Query Flow:**
```
SQL Query
    │
    ▼
Parse SQL (DataFusion sqlparser)
    │
    ▼
Analyze Query (extract tables, columns)
    │
    ▼
Create SessionContext with KylinCatalog
    │
    ▼
Execute Plan (DataFusion)
    │
    ▼
Collect RecordBatches
    │
    ▼
Format as QueryResult
```

### 6. kylin-engine
Build engine for creating pre-computed index layouts.

**Key Components:**
- `SegmentBuildJob` - Orchestrates segment building
- `FlatTableBuilder` - Joins fact and lookup tables
- `LayoutBuilder` - Computes aggregations
- `ParquetWriter` - Writes Parquet files

**Build Pipeline:**
```
Source Data
    │
    ▼
FlatTableBuilder.build()
    │ (Join fact + lookup tables)
    ▼
For Each Layout:
    │
    ├─► LayoutBuilder.build()
    │   ├─► GROUP BY dimensions
    │   ├─► Aggregate measures
    │   └─► Return RecordBatch
    │
    └─► ParquetWriter.write()
        └─► Write to storage
```

### 7. kylin-job
Job scheduling and management.

**Key Components:**
- `Job` - Job definition
- `JobStore` trait - Job persistence interface
- `InMemoryJobStore` - In-memory implementation
- `JobScheduler` - Job execution scheduler

### 8. kylin-api
REST API handlers using Axum.

**Endpoints:**
- `/api/models` - Model CRUD
- `/api/query` - Query execution
- `/api/projects` - Project management
- `/api/jobs` - Job management
- `/api/datasources` - Datasource management

### 9. kylin-server
Main server binary.

**Responsibilities:**
- Initialize stores and services
- Configure Axum router
- Serve static files
- Handle CORS and authentication

## Data Flow

### Query Execution
1. Client sends SQL query via REST API
2. `QueryExecutor` creates DataFusion `SessionContext`
3. `KylinCatalogProvider` provides model metadata
4. `KylinModelTableProvider` selects optimal layout
5. DataFusion executes query against pre-computed data
6. Results returned as `RecordBatch`
7. Formatted as JSON and returned to client

### Model Building
1. Client creates build job via REST API
2. `JobScheduler` picks up pending job
3. `SegmentBuildJob` orchestrates build:
   - `FlatTableBuilder` joins tables
   - `LayoutBuilder` computes aggregations
   - `ParquetWriter` writes results
4. Segment metadata updated
5. Job status updated

## Design Decisions

### Why Rust?
- **Performance**: Near-native performance for data processing
- **Memory Safety**: No garbage collector, no null pointer exceptions
- **Concurrency**: Async/await with Tokio for high concurrency
- **Ecosystem**: Rich ecosystem for data processing (Arrow, Parquet, DataFusion)

### Why DataFusion?
- **SQL Support**: Full SQL parser and optimizer
- **Extensibility**: Custom table providers and UDFs
- **Performance**: Optimized for analytical queries
- **Arrow Integration**: Native Arrow format support

### Why Axum?
- **Performance**: Built on Tokio for async I/O
- **Type Safety**: Type-safe extractors and handlers
- **Ecosystem**: Tower middleware ecosystem
- **Compatibility**: Works seamlessly with DataFusion

## Storage Architecture

### Metadata Storage
- **SQLite**: Default, embedded database for development
- **PostgreSQL**: Production database for multi-node deployments

### Data Storage
- **Local Filesystem**: Default for development
- **S3-Compatible**: For production cloud deployments

### Directory Structure
```
data/
├── {project}/
│   ├── {model_uuid}/
│   │   ├── {segment_uuid}/
│   │   │   ├── {layout_id}/
│   │   │   │   ├── data.parquet
│   │   │   │   └── ...
│   │   │   └── ...
│   │   └── ...
│   └── ...
└── ...
```

## Security

### Authentication
- Token-based authentication
- Session management
- Role-based access control

### Authorization
- Project-level permissions
- Model-level permissions
- Operation-level permissions

## Performance Considerations

### Query Performance
- Pre-computed layouts for fast query response
- Layout selection algorithm for optimal data access
- Filter pushdown for reduced data scanning

### Build Performance
- Parallel layout computation
- Efficient Parquet writing
- Incremental segment building

### Scalability
- Horizontal scaling with multiple server instances
- Shared metadata store (PostgreSQL)
- Distributed storage (S3)
