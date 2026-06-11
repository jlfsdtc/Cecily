# Kylin-Rust

A Rust implementation of Apache Kylin, replacing Spark with DataFusion and Java with Rust.

## Overview

Kylin-Rust is a high-performance OLAP analytics engine built with:
- **DataFusion** for query execution and build pipeline
- **Axum** for REST API
- **Arrow/Parquet** for columnar data storage
- **Tokio** for async runtime

## Architecture

```
kylin-rust/
├── crates/
│   ├── kylin-common/      # Shared types, config, errors
│   ├── kylin-metadata/    # Model, table, project metadata
│   ├── kylin-storage/     # Storage abstraction (Parquet, object store)
│   ├── kylin-catalog/     # DataFusion catalog/schema providers
│   ├── kylin-engine/      # Build engine (index computation)
│   ├── kylin-query/       # Query engine (layout matching, execution)
│   ├── kylin-job/         # Job scheduling and management
│   ├── kylin-api/         # Axum REST API handlers
│   └── kylin-server/      # Main binary (Axum server entry point)
├── kystudio/              # Vue frontend (unchanged from Kylin)
├── build/                 # Build scripts, config templates
├── tests/                 # Integration tests
└── docs/                  # Documentation
```

## Quick Start

### Prerequisites

- Rust 1.70+
- Node.js 16+ (for frontend)
- PostgreSQL or SQLite (for metadata storage)

### Build

```bash
# Build Rust backend
cargo build --release

# Build frontend (optional)
cd kystudio
npm install
npm run build
```

### Run

```bash
# Set environment variables
export KYLIN_SERVER_HOST=0.0.0.0
export KYLIN_SERVER_PORT=7070
export KYLIN_METADATA_DB_URL=sqlite:kylin.db
export KYLIN_DATA_DIR=./data

# Run server
cargo run --release --bin kylin-server
```

### Configuration

Configuration can be set via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `KYLIN_SERVER_HOST` | `0.0.0.0` | Server host |
| `KYLIN_SERVER_PORT` | `7070` | Server port |
| `KYLIN_METADATA_DB_URL` | `sqlite:kylin.db` | Database URL |
| `KYLIN_DATA_DIR` | `./data` | Data storage directory |
| `KYLIN_MAX_CONCURRENT_JOBS` | `4` | Max concurrent build jobs |
| `KYLIN_LOG_LEVEL` | `info` | Log level |

## API

The REST API is compatible with the original Kylin API. See [API Documentation](docs/api.md) for details.

### Key Endpoints

#### Models
- `GET /api/models?project={project}` - List models
- `POST /api/models` - Create model
- `GET /api/models/{model}` - Get model
- `PUT /api/models/{model}` - Update model
- `DELETE /api/models/{model}` - Delete model

#### Query
- `POST /api/query` - Execute SQL query

#### Projects
- `GET /api/projects` - List projects
- `POST /api/projects` - Create project
- `GET /api/projects/{project}` - Get project
- `PUT /api/projects/{project}` - Update project
- `DELETE /api/projects/{project}` - Delete project

#### Jobs
- `GET /api/jobs?project={project}` - List jobs
- `POST /api/jobs` - Create job
- `GET /api/jobs/{job}` - Get job
- `DELETE /api/jobs/{job}` - Delete job

## Development

### Running Tests

```bash
cargo test
```

### Code Quality

```bash
cargo clippy
cargo fmt
```

### Building Docker Image

```bash
docker build -t kylin-rust .
```

## Implementation Status

### Phase 0: Project Scaffolding ✅
- Cargo workspace with 9 crates
- Core type definitions
- Development tooling

### Phase 1: Metadata Layer ✅
- SQLite/PostgreSQL metadata store
- Model, Dataflow, Segment, Project CRUD
- MetadataManager for high-level operations

### Phase 2: DataFusion Catalog ✅
- KylinCatalogProvider
- KylinSchemaProvider
- KylinModelTableProvider
- Arrow schema conversion
- Layout selection engine
- Custom UDAFs (HLL COUNT DISTINCT)

### Phase 3: Query Engine ✅
- QueryExecutor with DataFusion
- OlapQueryContext
- QueryAnalyzer
- RecordBatch to QueryResult conversion

### Phase 4: Build Engine ✅
- FlatTableBuilder
- LayoutBuilder
- SegmentBuildJob
- ParquetWriter

### Phase 5: REST API ✅
- Model CRUD endpoints
- Query execution endpoint
- Project management endpoints
- Job management endpoints
- Datasource endpoints

### Phase 6: Frontend Integration ✅
- Main server implementation
- Static file serving
- CORS support
- Authentication middleware

## License

Apache License 2.0
