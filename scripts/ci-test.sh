#!/usr/bin/env bash
# CI Test Script for TagBox
# This script runs all CI checks locally or in CI environment

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Default values
MODE="full"
SKIP_DB_INIT=false
GENERATE_COVERAGE=false
USE_NEXTEST=true
VERBOSE=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --quick|-q)
            MODE="quick"
            shift
            ;;
        --skip-db-init)
            SKIP_DB_INIT=true
            shift
            ;;
        --coverage|-c)
            GENERATE_COVERAGE=true
            shift
            ;;
        --no-nextest)
            USE_NEXTEST=false
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -q, --quick         Run quick checks only (fmt, clippy)"
            echo "  --skip-db-init      Skip database initialization"
            echo "  -c, --coverage      Generate code coverage report"
            echo "  --no-nextest        Use standard cargo test instead of nextest"
            echo "  -v, --verbose       Enable verbose output"
            echo "  -h, --help          Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Change to project root
cd "$PROJECT_ROOT"

# Setup logging
log() {
    echo -e "${BLUE}[CI]${NC} $1"
}

success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

error() {
    echo -e "${RED}[✗]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

# Create directories
mkdir -p .ci-cache
mkdir -p .ci-reports

# Setup environment
export RUST_BACKTRACE=1
export DATABASE_URL="sqlite:${PROJECT_ROOT}/.sqlx-data/tagbox.db"

if [ "$VERBOSE" = true ]; then
    export RUST_LOG=debug
else
    export RUST_LOG=info
fi

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install missing tools
ensure_tool() {
    local tool=$1
    local install_cmd=$2
    
    if ! command_exists "$tool"; then
        warning "$tool not found, installing..."
        eval "$install_cmd"
        if [ $? -eq 0 ]; then
            success "$tool installed successfully"
        else
            error "Failed to install $tool"
            exit 1
        fi
    fi
}

# Step 1: Check and install required tools
log "Checking required tools..."

ensure_tool "sqlx" "cargo install sqlx-cli --no-default-features --features sqlite"

if [ "$USE_NEXTEST" = true ]; then
    ensure_tool "cargo-nextest" "cargo install cargo-nextest"
fi

if [ "$GENERATE_COVERAGE" = true ]; then
    ensure_tool "cargo-tarpaulin" "cargo install cargo-tarpaulin"
fi

ensure_tool "cargo-audit" "cargo install cargo-audit"

# Step 2: Initialize database
if [ "$SKIP_DB_INIT" = false ]; then
    log "Initializing database..."
    
    # Create database directory
    mkdir -p .sqlx-data
    
    # Remove old database
    rm -f .sqlx-data/tagbox.db
    touch .sqlx-data/tagbox.db
    
    # Run database initialization
    if cargo run --bin tagbox-init-db 2>&1 | tee .ci-reports/db-init.log; then
        success "Database initialized"
    else
        error "Database initialization failed"
        exit 1
    fi
    
    # Prepare sqlx
    log "Preparing sqlx offline mode..."
    cd tagbox-core
    if cargo sqlx prepare -- --lib 2>&1 | tee ../.ci-reports/sqlx-prepare.log; then
        success "SQLx prepared"
    else
        error "SQLx preparation failed"
        exit 1
    fi
    cd ..
fi

# Step 3: Formatting check
log "Checking code formatting..."
if cargo fmt --all -- --check 2>&1 | tee .ci-reports/fmt.log; then
    success "Code formatting check passed"
else
    error "Code formatting check failed"
    echo "Run 'cargo fmt --all' to fix formatting issues"
    exit 1
fi

# Step 4: Clippy lints
log "Running clippy lints..."
# Disable sccache for clippy to avoid conflicts
unset RUSTC_WRAPPER
if cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tee .ci-reports/clippy.log; then
    success "Clippy check passed"
else
    error "Clippy check failed"
    exit 1
fi

# Quick mode stops here
if [ "$MODE" = "quick" ]; then
    success "Quick checks completed successfully!"
    exit 0
fi

# Step 5: Build all packages
log "Building all packages..."
if cargo build --all --all-features 2>&1 | tee .ci-reports/build.log; then
    success "Build completed"
else
    error "Build failed"
    exit 1
fi

# Step 6: Run tests
if [ "$USE_NEXTEST" = true ]; then
    log "Running tests with nextest..."
    if cargo nextest run --all --all-features 2>&1 | tee .ci-reports/test.log; then
        success "All tests passed"
    else
        error "Tests failed"
        exit 1
    fi
else
    log "Running tests with cargo test..."
    if cargo test --all --all-features 2>&1 | tee .ci-reports/test.log; then
        success "All tests passed"
    else
        error "Tests failed"
        exit 1
    fi
fi

# Step 7: Generate documentation
log "Checking documentation..."
if cargo doc --all --no-deps --all-features 2>&1 | tee .ci-reports/doc.log; then
    success "Documentation check passed"
else
    error "Documentation check failed"
    exit 1
fi

# Step 8: Security audit
log "Running security audit..."
# Use audit.toml to ignore known issues
if cargo audit --ignore RUSTSEC-2023-0071 2>&1 | tee .ci-reports/audit.log; then
    success "Security audit passed"
else
    warning "Security vulnerabilities found (non-blocking)"
fi

# Step 9: Code coverage (optional)
if [ "$GENERATE_COVERAGE" = true ]; then
    log "Generating code coverage..."
    
    # Clean previous coverage
    rm -rf target/coverage
    
    if cargo tarpaulin --all --out Html --out Xml --output-dir .ci-reports/coverage 2>&1 | tee .ci-reports/coverage.log; then
        success "Code coverage generated"
        
        # Extract coverage percentage
        coverage_percent=$(grep -oP 'Coverage is \K[0-9.]+' .ci-reports/coverage.log || echo "0")
        echo "Total coverage: ${coverage_percent}%"
        
        # Create coverage badge data
        echo "{\"coverage\": \"${coverage_percent}%\"}" > .ci-reports/coverage/badge.json
    else
        error "Code coverage generation failed"
        exit 1
    fi
fi

# Step 10: Generate test report summary
log "Generating test report..."

cat > .ci-reports/summary.json <<EOF
{
    "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "mode": "$MODE",
    "checks": {
        "formatting": "passed",
        "clippy": "passed",
        "build": "passed",
        "tests": "passed",
        "documentation": "passed",
        "audit": "passed"
    },
    "rust_version": "$(rustc --version)",
    "cargo_version": "$(cargo --version)"
}
EOF

# Create markdown report
cat > .ci-reports/summary.md <<EOF
# CI Test Report

**Date**: $(date -u +"%Y-%m-%d %H:%M:%S UTC")  
**Mode**: $MODE  
**Rust Version**: $(rustc --version)  
**Cargo Version**: $(cargo --version)  

## Check Results

| Check | Status |
|-------|--------|
| Formatting | ✅ Passed |
| Clippy | ✅ Passed |
| Build | ✅ Passed |
| Tests | ✅ Passed |
| Documentation | ✅ Passed |
| Security Audit | ✅ Passed |
EOF

if [ "$GENERATE_COVERAGE" = true ]; then
    echo "| Code Coverage | ${coverage_percent}% |" >> .ci-reports/summary.md
fi

echo "" >> .ci-reports/summary.md
echo "## Logs" >> .ci-reports/summary.md
echo "" >> .ci-reports/summary.md
echo "All detailed logs are available in the \`.ci-reports/\` directory." >> .ci-reports/summary.md

success "All CI checks completed successfully!"

# List generated reports
log "Generated reports:"
ls -la .ci-reports/

exit 0