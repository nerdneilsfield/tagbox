#!/usr/bin/env bash
set -euo pipefail

# Script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
export DATABASE_URL="sqlite:${PROJECT_ROOT}/.sqlx-data/tagbox.db"


echo "=== Building project ==="

# Build all packages
echo "Building all packages..."
cargo build --all --verbose

echo "=== Build completed successfully ==="