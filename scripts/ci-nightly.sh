#!/usr/bin/env bash
set -euo pipefail

echo "=== Running Nightly Tests ==="

# Setup environment
export RUST_BACKTRACE=1
# Script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
export DATABASE_URL="sqlite:${PROJECT_ROOT}/.sqlx-data/tagbox.db"

# Initialize database
echo "Initializing test database..."
mkdir -p .sqlx-data
rm -f .sqlx-data/tagbox.db
touch .sqlx-data/tagbox.db
cargo run --bin tagbox-init-db

# Prepare sqlx
echo "Preparing sqlx offline mode..."
cd tagbox-core
cargo sqlx prepare -- --lib
cd ..

# Run comprehensive tests
echo "Running comprehensive test suite..."
cargo test --all --all-features

# Check for future compatibility
echo "Checking for future compatibility warnings..."
cargo check --all --all-features

# Run benchmarks if they exist
if cargo bench --no-run 2>/dev/null; then
    echo "Running benchmarks..."
    cargo bench
fi

# Security audit with detailed report
echo "Running detailed security audit..."
cargo audit --ignore RUSTSEC-2023-0071

echo "=== Nightly tests completed ==="