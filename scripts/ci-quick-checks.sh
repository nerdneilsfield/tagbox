#!/usr/bin/env bash
set -euo pipefail

# Script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
export DATABASE_URL="sqlite:${PROJECT_ROOT}/.sqlx-data/tagbox.db"

echo "=== Running Quick Checks ==="
export 

# Check formatting
echo "Checking code formatting..."
cargo fmt --all -- --check

# Run clippy
echo "Running clippy..."
# 避免 sccache 和 clippy 冲突，临时禁用 RUSTC_WRAPPER
unset RUSTC_WRAPPER
cargo clippy --all-targets --all-features -- -D warnings

echo "=== Quick checks passed! ==="