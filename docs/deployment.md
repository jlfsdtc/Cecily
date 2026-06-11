# Kylin-Rust Deployment Guide

## Prerequisites

### System Requirements

- **CPU**: 4+ cores recommended
- **RAM**: 8GB+ recommended
- **Disk**: 50GB+ for data storage
- **OS**: Linux (Ubuntu 20.04+, CentOS 7+), macOS

### Software Requirements

- **Rust**: 1.70+ (for building from source)
- **Node.js**: 16+ (for frontend build)
- **Database**: SQLite (default) or PostgreSQL 12+
- **Docker**: 20.10+ (for container deployment)
- **Kubernetes**: 1.20+ (for K8s deployment)

## Build from Source

### 1. Clone Repository

```bash
git clone https://github.com/apache/kylin-rust.git
cd kylin-rust
```

### 2. Build Backend

```bash
cargo build --release
```

### 3. Build Frontend (Optional)

```bash
cd kystudio
npm install
npm run build
cd ..
```

### 4. Run Tests

```bash
cargo test
```

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `KYLIN_SERVER_HOST` | `0.0.0.0` | Server bind address |
| `KYLIN_SERVER_PORT` | `7070` | Server port |
| `KYLIN_METADATA_DB_URL` | `sqlite:kylin.db` | Database URL |
| `KYLIN_DATA_DIR` | `./data` | Data storage directory |
| `KYLIN_MAX_CONCURRENT_JOBS` | `4` | Max concurrent build jobs |
| `KYLIN_LOG_LEVEL` | `info` | Log level (trace, debug, info, warn, error) |

### Database Configuration

#### SQLite (Default)

```bash
export KYLIN_METADATA_DB_URL=sqlite:kylin.db
```

#### PostgreSQL

```bash
export KYLIN_METADATA_DB_URL=postgres://user:password@localhost:5432/kylin
```

### Storage Configuration

#### Local Filesystem (Default)

```bash
export KYLIN_DATA_DIR=/var/lib/kylin/data
```

#### S3-Compatible Storage

```bash
export KYLIN_DATA_DIR=s3://my-bucket/kylin/data
```

## Running Locally

### 1. Set Environment Variables

```bash
export KYLIN_SERVER_HOST=0.0.0.0
export KYLIN_SERVER_PORT=7070
export KYLIN_METADATA_DB_URL=sqlite:kylin.db
export KYLIN_DATA_DIR=./data
```

### 2. Start Server

```bash
cargo run --release --bin kylin-server
```

### 3. Verify Server

```bash
curl http://localhost:7070/api/projects
```

### 4. Access Frontend

Open browser: `http://localhost:7070/`

## Docker Deployment

### 1. Build Docker Image

```bash
docker build -t kylin-rust .
```

### 2. Run Container

```bash
docker run -d \
  --name kylin \
  -p 7070:7070 \
  -v kylin-data:/app/data \
  -e KYLIN_SERVER_HOST=0.0.0.0 \
  -e KYLIN_SERVER_PORT=7070 \
  -e KYLIN_METADATA_DB_URL=sqlite:/app/data/kylin.db \
  -e KYLIN_DATA_DIR=/app/data \
  kylin-rust
```

### 3. Docker Compose

```bash
docker-compose up -d
```

## Kubernetes Deployment

### 1. Create Namespace

```bash
kubectl create namespace kylin
```

### 2. Create ConfigMap

```bash
kubectl apply -f k8s/configmap.yaml
```

### 3. Create Deployment

```bash
kubectl apply -f k8s/deployment.yaml
```

### 4. Create Service

```bash
kubectl apply -f k8s/service.yaml
```

### 5. Create Ingress (Optional)

```bash
kubectl apply -f k8s/ingress.yaml
```

### 6. Verify Deployment

```bash
kubectl get pods -n kylin
kubectl get services -n kylin
```

## Production Considerations

### 1. Database

- Use PostgreSQL for production
- Configure connection pooling
- Set up regular backups

### 2. Storage

- Use S3-compatible storage for scalability
- Configure appropriate retention policies
- Monitor storage usage

### 3. Monitoring

- Enable logging with appropriate level
- Set up metrics collection
- Configure alerting

### 4. Security

- Enable authentication
- Configure CORS appropriately
- Use HTTPS in production
- Set up firewall rules

### 5. Performance

- Tune `KYLIN_MAX_CONCURRENT_JOBS` based on CPU cores
- Allocate sufficient memory for DataFusion
- Monitor query performance

## Troubleshooting

### Server Won't Start

1. Check port availability: `lsof -i :7070`
2. Check database connection
3. Check log output for errors

### Query Failures

1. Verify model exists
2. Check data availability
3. Review query syntax

### Build Failures

1. Check disk space
2. Verify source data access
3. Review job logs

## Backup and Recovery

### Database Backup

#### SQLite

```bash
cp kylin.db kylin.db.backup
```

#### PostgreSQL

```bash
pg_dump -U kylin kylin > kylin.backup
```

### Data Backup

```bash
tar -czf kylin-data-backup.tar.gz /var/lib/kylin/data
```

### Recovery

#### SQLite

```bash
cp kylin.db.backup kylin.db
```

#### PostgreSQL

```bash
psql -U kylin kylin < kylin.backup
```

## Upgrading

### 1. Stop Server

```bash
# Docker
docker stop kylin

# Kubernetes
kubectl scale deployment kylin --replicas=0 -n kylin
```

### 2. Backup Data

```bash
# Follow backup instructions above
```

### 3. Build New Version

```bash
git pull
cargo build --release
```

### 4. Start Server

```bash
# Docker
docker start kylin

# Kubernetes
kubectl scale deployment kylin --replicas=1 -n kylin
```

## Support

For issues and questions:
- GitHub Issues: https://github.com/apache/kylin-rust/issues
- Documentation: https://kylin.apache.org/docs/
