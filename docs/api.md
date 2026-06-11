# Kylin-Rust API Reference

## Overview

Kylin-Rust provides a RESTful API compatible with the original Apache Kylin API. All endpoints return JSON responses in a consistent format.

## Base URL

```
http://localhost:7070/api
```

## Response Format

All responses follow this format:

```json
{
    "code": "000",
    "data": { ... },
    "msg": ""
}
```

### Success Response
- `code`: "000" for success
- `data`: Response data
- `msg`: Empty string

### Error Response
- `code`: HTTP status code as string
- `data`: null
- `msg`: Error message

## Authentication

All endpoints require authentication via Bearer token:

```
Authorization: Bearer <token>
```

## Projects

### List Projects

```
GET /api/projects
```

**Response:**
```json
{
    "code": "000",
    "data": {
        "projects": [
            {
                "uuid": "...",
                "name": "my_project",
                "description": "My project",
                "default_database": "DEFAULT",
                "active": true,
                "last_modified": 1234567890,
                "version": 1
            }
        ],
        "size": 1
    }
}
```

### Create Project

```
POST /api/projects
```

**Request:**
```json
{
    "name": "my_project",
    "description": "My project",
    "default_database": "DEFAULT"
}
```

**Response:**
```json
{
    "code": "000",
    "data": {
        "uuid": "...",
        "name": "my_project",
        "description": "My project",
        "default_database": "DEFAULT",
        "active": true,
        "last_modified": 1234567890,
        "version": 1
    }
}
```

### Get Project

```
GET /api/projects/{project}
```

**Response:**
```json
{
    "code": "000",
    "data": {
        "uuid": "...",
        "name": "my_project",
        "description": "My project",
        "default_database": "DEFAULT",
        "active": true,
        "last_modified": 1234567890,
        "version": 1
    }
}
```

### Update Project

```
PUT /api/projects/{project}
```

**Request:**
```json
{
    "description": "Updated description",
    "default_database": "MY_DB"
}
```

**Response:**
```json
{
    "code": "000",
    "data": {
        "uuid": "...",
        "name": "my_project",
        "description": "Updated description",
        "default_database": "MY_DB",
        "active": true,
        "last_modified": 1234567890,
        "version": 2
    }
}
```

### Delete Project

```
DELETE /api/projects/{project}
```

**Response:**
```json
{
    "code": "000",
    "data": {}
}
```

## Models

### List Models

```
GET /api/models?project={project}
```

**Query Parameters:**
- `project` (required): Project name

**Response:**
```json
{
    "code": "000",
    "data": {
        "models": [
            {
                "uuid": "...",
                "name": "sales_model",
                "root_fact_table": "DEFAULT.KYLIN_SALES",
                "model_type": "Batch",
                "join_tables": [],
                "all_columns": [],
                "all_measures": [],
                "filter_condition": null,
                "partition_desc": null,
                "computed_columns": [],
                "last_modified": 1234567890,
                "version": 1
            }
        ],
        "size": 1
    }
}
```

### Create Model

```
POST /api/models
```

**Request:**
```json
{
    "project": "my_project",
    "name": "sales_model",
    "root_fact_table": "DEFAULT.KYLIN_SALES",
    "model_type": "Batch"
}
```

**Response:**
```json
{
    "code": "000",
    "data": {
        "uuid": "...",
        "name": "sales_model",
        "root_fact_table": "DEFAULT.KYLIN_SALES",
        "model_type": "Batch",
        "join_tables": [],
        "all_columns": [],
        "all_measures": [],
        "filter_condition": null,
        "partition_desc": null,
        "computed_columns": [],
        "last_modified": 1234567890,
        "version": 1
    }
}
```

### Get Model

```
GET /api/models/{model}
```

**Response:**
```json
{
    "code": "000",
    "data": {
        "uuid": "...",
        "name": "sales_model",
        "root_fact_table": "DEFAULT.KYLIN_SALES",
        "model_type": "Batch",
        "join_tables": [],
        "all_columns": [],
        "all_measures": [],
        "filter_condition": null,
        "partition_desc": null,
        "computed_columns": [],
        "last_modified": 1234567890,
        "version": 1
    }
}
```

### Update Model

```
PUT /api/models/{model}
```

**Request:**
```json
{
    "name": "updated_model",
    "root_fact_table": "DEFAULT.NEW_TABLE",
    "model_type": "Batch",
    "filter_condition": "amount > 100"
}
```

**Response:**
```json
{
    "code": "000",
    "data": {
        "uuid": "...",
        "name": "updated_model",
        "root_fact_table": "DEFAULT.NEW_TABLE",
        "model_type": "Batch",
        "filter_condition": "amount > 100",
        "last_modified": 1234567890,
        "version": 2
    }
}
```

### Delete Model

```
DELETE /api/models/{model}
```

**Response:**
```json
{
    "code": "000",
    "data": {}
}
```

### Rename Model

```
PUT /api/models/{model}/name
```

**Request:**
```json
{
    "name": "new_name"
}
```

**Response:**
```json
{
    "code": "000",
    "data": {
        "uuid": "...",
        "name": "new_name",
        ...
    }
}
```

### Clone Model

```
POST /api/models/{model}/clone
```

**Response:**
```json
{
    "code": "000",
    "data": {
        "uuid": "...",
        "name": "sales_model_clone",
        ...
    }
}
```

## Query

### Execute Query

```
POST /api/query
```

**Request:**
```json
{
    "sql": "SELECT COUNT(*) FROM sales_model",
    "project": "my_project"
}
```

**Response:**
```json
{
    "code": "000",
    "data": {
        "columnMetas": [
            {
                "name": "COUNT(*)",
                "dataType": "Int64"
            }
        ],
        "results": [
            [1000]
        ],
        "duration": 50,
        "totalScanRows": 1000,
        "totalScanBytes": 1024000,
        "totalRemainingRows": 1
    }
}
```

## Jobs

### List Jobs

```
GET /api/jobs?project={project}&limit={limit}
```

**Query Parameters:**
- `project` (required): Project name
- `limit` (optional): Maximum number of jobs (default: 100)

**Response:**
```json
{
    "code": "000",
    "data": {
        "jobs": [
            {
                "uuid": "...",
                "project": "my_project",
                "job_type": "SegmentBuild",
                "status": "Finished",
                "params": {
                    "model_uuid": "...",
                    "segment_id": "...",
                    "time_range": [1000000, 2000000],
                    "layout_ids": [1, 2, 3]
                },
                "result": {
                    "segment_id": "...",
                    "rows_affected": 10000,
                    "duration_ms": 5000
                },
                "progress": 1.0,
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:05Z",
                "error_message": null
            }
        ],
        "size": 1
    }
}
```

### Create Job

```
POST /api/jobs
```

**Request:**
```json
{
    "project": "my_project",
    "job_type": "SegmentBuild",
    "model_uuid": "...",
    "segment_id": "...",
    "time_range": [1000000, 2000000],
    "layout_ids": [1, 2, 3]
}
```

**Response:**
```json
{
    "code": "000",
    "data": {
        "uuid": "...",
        "project": "my_project",
        "job_type": "SegmentBuild",
        "status": "Pending",
        "params": {
            "model_uuid": "...",
            "segment_id": "...",
            "time_range": [1000000, 2000000],
            "layout_ids": [1, 2, 3]
        },
        "result": null,
        "progress": 0.0,
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z",
        "error_message": null
    }
}
```

### Get Job

```
GET /api/jobs/{job}
```

**Response:**
```json
{
    "code": "000",
    "data": {
        "uuid": "...",
        "project": "my_project",
        "job_type": "SegmentBuild",
        "status": "Running",
        "params": { ... },
        "result": null,
        "progress": 0.5,
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:02Z",
        "error_message": null
    }
}
```

### Get Job Status

```
GET /api/jobs/{job}/status
```

**Response:**
```json
{
    "code": "000",
    "data": {
        "uuid": "...",
        "status": "Running",
        "progress": 0.5,
        "error_message": null
    }
}
```

### Delete Job

```
DELETE /api/jobs/{job}
```

**Response:**
```json
{
    "code": "000",
    "data": {}
}
```

## Datasources

### List Datasources

```
GET /api/datasources
```

**Response:**
```json
{
    "code": "000",
    "data": {
        "datasources": [
            {
                "name": "hive",
                "source_type": "HIVE",
                "connection_url": "thrift://localhost:9083"
            }
        ],
        "size": 1
    }
}
```

### Add Datasource

```
POST /api/datasources
```

**Request:**
```json
{
    "name": "my_hive",
    "source_type": "HIVE",
    "connection_url": "thrift://localhost:9083",
    "properties": {
        "database": "default"
    }
}
```

**Response:**
```json
{
    "code": "000",
    "data": {
        "name": "my_hive",
        "source_type": "HIVE"
    }
}
```

### Get Source Tables

```
GET /api/datasources/{source}/tables
```

**Response:**
```json
{
    "code": "000",
    "data": {
        "tables": [
            {
                "name": "sales",
                "database": "default",
                "type": "TABLE"
            }
        ],
        "size": 1
    }
}
```

### Get Source Table

```
GET /api/datasources/{source}/tables/{table}
```

**Response:**
```json
{
    "code": "000",
    "data": {
        "name": "sales",
        "database": "default",
        "columns": [
            {
                "name": "id",
                "type": "bigint"
            },
            {
                "name": "amount",
                "type": "double"
            }
        ]
    }
}
```

### Build Snapshot

```
POST /api/datasources/{source}/tables/{table}/snapshot
```

**Response:**
```json
{
    "code": "000",
    "data": {
        "status": "submitted",
        "source": "my_hive",
        "table": "sales"
    }
}
```

## Error Codes

| Code | Description |
|------|-------------|
| 200 | OK |
| 400 | Bad Request |
| 401 | Unauthorized |
| 403 | Forbidden |
| 404 | Not Found |
| 500 | Internal Server Error |

## Examples

### Create Project and Model

```bash
# Create project
curl -X POST http://localhost:7070/api/projects \
  -H "Content-Type: application/json" \
  -d '{"name": "my_project", "description": "My project"}'

# Create model
curl -X POST http://localhost:7070/api/models \
  -H "Content-Type: application/json" \
  -d '{"project": "my_project", "name": "sales_model", "root_fact_table": "DEFAULT.SALES"}'

# Execute query
curl -X POST http://localhost:7070/api/query \
  -H "Content-Type: application/json" \
  -d '{"sql": "SELECT COUNT(*) FROM sales_model", "project": "my_project"}'
```
