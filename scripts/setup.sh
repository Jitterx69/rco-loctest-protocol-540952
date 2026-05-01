#!/bin/bash
set -e

# RCO Setup Script
# Purpose: Checks the environment and prepares the system.

echo "--- RCO Environment Setup ---"

# 1. Check for Julia
if ! command -v julia &> /dev/null; then
    echo "Error: Julia is not installed."
    echo "Run: curl -fsSL https://install.julialang.org | sh"
    exit 1
fi
JULIA_VERSION=$(julia --version)
echo "Found $JULIA_VERSION"

# 2. Check/Install Julia Dependencies
echo "Checking dependencies..."
julia -e 'using Pkg; "JSON" in keys(Pkg.project().dependencies) || (println("Installing JSON..."); Pkg.add("JSON"))'

# 3. Verify Binary
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DIST_DIR="${SCRIPT_DIR}/../dist"
LIB_PATH="${DIST_DIR}/librco.so"

if [ ! -f "$LIB_PATH" ]; then
    echo "Error: librco.so not found in $DIST_DIR"
    exit 1
fi

# 3. Verify Manifest
MANIFEST="${DIST_DIR}/manifest.json"
if [ -f "$MANIFEST" ]; then
    EXPECTED_HASH=$(grep -oP '"hash": "\K[^"]+' "$MANIFEST")
    ACTUAL_HASH=$(sha256sum "$LIB_PATH" | awk '{print $1}')
    
    if [ "$EXPECTED_HASH" == "$ACTUAL_HASH" ]; then
        echo "Manifest: OK"
    else
        echo "Error: Hash mismatch!"
    fi
fi

echo "--- Setup Complete. Use 'julia scripts/start.jl' to begin ---"
