#!/bin/bash

# Kylin-Rust Build Script

set -e

echo "Building Kylin-Rust..."

# Build Rust backend
echo "Building Rust backend..."
cargo build --release

# Build frontend (if kystudio directory exists)
if [ -d "kystudio" ]; then
    echo "Building frontend..."
    cd kystudio
    npm install
    npm run build
    cd ..
else
    echo "Frontend directory not found, skipping..."
fi

echo "Build complete!"
echo ""
echo "To run the server:"
echo "  cargo run --release --bin kylin-server"
echo ""
echo "Or set environment variables:"
echo "  export KYLIN_SERVER_HOST=0.0.0.0"
echo "  export KYLIN_SERVER_PORT=7070"
echo "  export KYLIN_METADATA_DB_URL=sqlite:kylin.db"
echo "  export KYLIN_DATA_DIR=./data"
echo "  cargo run --release --bin kylin-server"
