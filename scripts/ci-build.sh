#!/usr/bin/env bash
set -euo pipefail

echo "=== Building project ==="

# Build all packages
echo "Building all packages..."
cargo build --all --verbose

echo "=== Build completed successfully ==="