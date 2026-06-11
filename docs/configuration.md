# Kylin-Rust Configuration Reference

## Overview

Kylin-Rust is configured via environment variables. This document describes all available configuration options.

## Server Configuration

### KYLIN_SERVER_HOST

- **Type**: String
- **Default**: `0.0.0.0`
- **Description**: Server bind address

```bash
export KYLIN_SERVER_HOST=0.0.0.0
```

### KYLIN_SERVER_PORT

- **Type**: Integer
- **Default**: `7070`
- **Description**: Server port

```bash
export KYLIN_SERVER_PORT=7070
```

## Database Configuration

### KYLIN_METADATA_DB_URL

- **Type**: String
- **Default**: `sqlite:kylin.db`
- **Description**: Database connection URL

**SQLite:**
```bash
export KYLIN_METADATA_DB_URL=sqlite:kylin.db
export KYLIN_METADATA_DB_URL=sqlite:/var/lib/kylin/kylin.db
```

**PostgreSQL:**
```bash
export KYLIN_METADATA_DB_URL=postgres://user:password@localhost:5432/kylin
export KYLIN_METADATA_DB_URL=postgres://user:password@db-host:5432/kylin?sslmode=require
```

## Storage Configuration

### KYLIN_DATA_DIR

- **Type**: String
- **Default**: `./data`
- **Description**: Data storage directory

**Local Filesystem:**
```bash
export KYLIN_DATA_DIR=./data
export KYLIN_DATA_DIR=/var/lib/kylin/data
```

**S3-Compatible Storage:**
```bash
export KYLIN_DATA_DIR=s3://my-bucket/kylin/data
```

## Job Configuration

### KYLIN_MAX_CONCURRENT_JOBS

- **Type**: Integer
- **Default**: `4`
- **Description**: Maximum number of concurrent build jobs

```bash
export KYLIN_MAX_CONCURRENT_JOBS=8
```

## Logging Configuration

### KYLIN_LOG_LEVEL

- **Type**: String
- **Default**: `info`
- **Description**: Log level
- **Values**: `trace`, `debug`, `info`, `warn`, `error`

```bash
export KYLIN_LOG_LEVEL=info
export KYLIN_LOG_LEVEL=debug
```

## Configuration Examples

### Development

```bash
export KYLIN_SERVER_HOST=127.0.0.1
export KYLIN_SERVER_PORT=7070
export KYLIN_METADATA_DB_URL=sqlite:kylin.db
export KYLIN_DATA_DIR=./data
export KYLIN_MAX_CONCURRENT_JOBS=2
export KYLIN_LOG_LEVEL=debug
```

### Production

```bash
export KYLIN_SERVER_HOST=0.0.0.0
export KYLIN_SERVER_PORT=7070
export KYLIN_METADATA_DB_URL=postgres://kylin:password@db-host:5432/kylin
export KYLIN_DATA_DIR=/var/lib/kylin/data
export KYLIN_MAX_CONCURRENT_JOBS=8
export KYLIN_LOG_LEVEL=info
```

### Docker

```bash
docker run -d \
  --name kylin \
  -p 7070:7070 \
  -e KYLIN_SERVER_HOST=0.0.0.0 \
  -e KYLIN_SERVER_PORT=7070 \
  -e KYLIN_METADATA_DB_URL=sqlite:/app/data/kylin.db \
  -e KYLIN_DATA_DIR=/app/data \
  -e KYLIN_MAX_CONCURRENT_JOBS=4 \
  -e KYLIN_LOG_LEVEL=info \
  -v kylin-data:/app/data \
  kylin-rust
```

### Kubernetes

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: kylin-config
  namespace: kylin
data:
  KYLIN_SERVER_HOST: "0.0.0.0"
  KYLIN_SERVER_PORT: "7070"
  KYLIN_METADATA_DB_URL: "postgres://kylin:password@postgres:5432/kylin"
  KYLIN_DATA_DIR: "/app/data"
  KYLIN_MAX_CONCURRENT_JOBS: "4"
  KYLIN_LOG_LEVEL: "info"
```

## Database Setup

### SQLite

No setup required. Database file is created automatically.

### PostgreSQL

1. Create database:
```sql
CREATE DATABASE kylin;
CREATE USER kylin WITH PASSWORD 'password';
GRANT ALL PRIVILEGES ON DATABASE kylin TO kylin;
```

2. Tables are created automatically on first run.

## Storage Setup

### Local Filesystem

Create data directory:
```bash
mkdir -p /var/lib/kylin/data
chown -R kylin:kylin /var/lib/kylin
```

### S3-Compatible Storage

Configure AWS credentials:
```bash
export AWS_ACCESS_KEY_ID=your_access_key
export AWS_SECRET_ACCESS_KEY=your_secret_key
export AWS_REGION=us-east-1
```

Create bucket:
```bash
aws s3 mb s3://my-bucket
```

## Performance Tuning

### Memory

DataFusion uses memory for query execution. Allocate sufficient memory:

```bash
# For development
export RUST_MIN_STACK=8388608

# For production
export RUST_MIN_STACK=16777216
```

### CPU

Set `KYLIN_MAX_CONCURRENT_JOBS` based on CPU cores:

```bash
# For 4-core machine
export KYLIN_MAX_CONCURRENT_JOBS=4

# For 8-core machine
export KYLIN_MAX_CONCURRENT_JOBS=8
```

### Disk

For local storage, use SSD for better performance:

```bash
export KYLIN_DATA_DIR=/mnt/ssd/kylin/data
```

## Security Configuration

### Authentication

Enable authentication by setting a secret key:

```bash
export KYLIN_SECRET_KEY=your-secret-key
```

### CORS

Configure allowed origins:

```bash
export KYLIN_CORS_ORIGINS=http://localhost:3000,https://mydomain.com
```

### HTTPS

Use reverse proxy (nginx, caddy) for HTTPS termination.

## Monitoring

### Health Check

```bash
curl http://localhost:7070/api/health
```

### Metrics

Enable metrics endpoint:

```bash
export KYLIN_METRICS_ENABLED=true
export KYLIN_METRICS_PORT=9090
```

## Troubleshooting

### Database Connection Issues

1. Check database URL format
2. Verify database is running
3. Check network connectivity
4. Verify credentials

### Storage Issues

1. Check directory permissions
2. Verify disk space
3. Check S3 credentials (if using S4)

### Performance Issues

1. Increase `KYLIN_MAX_CONCURRENT_JOBS`
2. Allocate more memory
3. Use SSD storage
4. Check query complexity
