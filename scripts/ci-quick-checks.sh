#!/usr/bin/env bash
set -euo pipefail

echo "=== Running Quick Checks ==="

# Check formatting
echo "Checking code formatting..."
cargo fmt --all -- --check

# Run clippy
echo "Running clippy..."
# 避免 sccache 和 clippy 冲突，临时禁用 RUSTC_WRAPPER
unset RUSTC_WRAPPER
cargo clippy --all-targets --all-features -- -D warnings

echo "=== Quick checks passed! ==="