#!/bin/bash
set -e

# RCO Universal Shipment Script
# Consolidates all logic into a single "Universal Substrate" bundle.

PROJECT_ROOT=$(pwd)
TARGET_DIR="${PROJECT_ROOT}/target/release"
DIST_DIR="${PROJECT_ROOT}/dist"

echo "--- Building RCO Universal Release ---"

# 1. Compile Kernels
cargo build --release -p rco-sdk-julia

# 2. Copy Binary
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    LIB_NAME="librco_sdk_julia.so"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    LIB_NAME="librco_sdk_julia.dylib"
else
    LIB_NAME="rco_sdk_julia.dll"
fi
cp "${TARGET_DIR}/${LIB_NAME}" "${DIST_DIR}/librco.so"

# 3. Create Manifest
cat <<EOF > "${DIST_DIR}/manifest.json"
{
  "project": "RCO",
  "version": "5.0.0",
  "date": "$(date -u +'%Y-%m-%dT%H:%M:%SZ')",
  "hash": "$(sha256sum "${DIST_DIR}/librco.so" | awk '{print $1}')"
}
EOF

# 4. Create Universal Tarball
echo "Packaging rco-v5.tar.gz..."
tar -czf "${DIST_DIR}/rco-v5.tar.gz" \
    dist/librco.so \
    dist/librco.h \
    dist/manifest.json \
    bindings/ \
    scripts/start.jl \
    scripts/setup.sh

# 5. Build Docker Image (Optional)
if command -v docker &> /dev/null; then
    echo "Building Docker image rco-v5..."
    # Hide docker errors if daemon is down, doesn't block the tarball
    docker build -t rco-v5 . || echo "Docker build failed, skipping..."
fi

echo "--- Done. Universal Bundle ready in ${DIST_DIR} ---"
