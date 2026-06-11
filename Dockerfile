# Multi-stage build for Kylin-Rust

# Stage 1: Build Rust backend
FROM rust:1.82 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
RUN cargo build --release

# Stage 2: Build frontend (optional)
FROM node:18 AS frontend
WORKDIR /app
COPY kystudio/ .
RUN npm install && npm run build

# Stage 3: Production image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Rust binary
COPY --from=builder /app/target/release/kylin-server /usr/local/bin/

# Copy frontend build (if available)
COPY --from=frontend /app/dist /app/static

# Create data directory
RUN mkdir -p /app/data

# Create non-root user
RUN useradd -m -u 1000 kylin && chown -R kylin:kylin /app
USER kylin

# Expose port
EXPOSE 7070

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:7070/api/health || exit 1

# Set environment variables
ENV KYLIN_SERVER_HOST=0.0.0.0
ENV KYLIN_SERVER_PORT=7070
ENV KYLIN_METADATA_DB_URL=sqlite:/app/data/kylin.db
ENV KYLIN_DATA_DIR=/app/data
ENV KYLIN_LOG_LEVEL=info

# Run server
CMD ["kylin-server"]
