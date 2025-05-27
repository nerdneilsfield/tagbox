#!/bin/bash

# TagBox Database Initialization Script
# This script initializes a TagBox database using the SQL schema

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
SQL_FILE="$SCRIPT_DIR/init-database.sql"

# Default values
DB_PATH=""
FORCE=false
VERBOSE=false

# Help function
show_help() {
    cat << EOF
TagBox Database Initialization Script

USAGE:
    $0 [OPTIONS] <database_path>

ARGUMENTS:
    <database_path>    Path to the SQLite database file (will be created if doesn't exist)

OPTIONS:
    -f, --force       Force recreate database (will delete existing data!)
    -v, --verbose     Enable verbose output
    -h, --help        Show this help message

EXAMPLES:
    # Initialize a new database
    $0 ./tagbox.db
    
    # Initialize with verbose output
    $0 -v ./data/tagbox.db
    
    # Force recreate existing database
    $0 --force ./tagbox.db
    
    # Using DATABASE_URL environment variable
    export DATABASE_URL="sqlite:./tagbox.db"
    $0 \${DATABASE_URL#sqlite:}

NOTES:
    - The script requires sqlite3 command-line tool
    - Database file and parent directories will be created if they don't exist
    - Use --force carefully as it will delete all existing data
EOF
}

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

# Check if sqlite3 is available
check_sqlite3() {
    if ! command -v sqlite3 &> /dev/null; then
        log_error "sqlite3 command not found!"
        log_error "Please install SQLite3:"
        log_error "  - macOS: brew install sqlite3"
        log_error "  - Ubuntu/Debian: sudo apt install sqlite3"
        log_error "  - CentOS/RHEL: sudo yum install sqlite"
        exit 1
    fi
    
    local version
    version=$(sqlite3 -version | cut -d' ' -f1)
    if [[ $VERBOSE == true ]]; then
        log_info "Using SQLite version: $version"
    fi
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -f|--force)
                FORCE=true
                shift
                ;;
            -v|--verbose)
                VERBOSE=true
                shift
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            -*)
                log_error "Unknown option: $1"
                echo
                show_help
                exit 1
                ;;
            *)
                if [[ -z "$DB_PATH" ]]; then
                    DB_PATH="$1"
                else
                    log_error "Too many arguments. Expected one database path."
                    exit 1
                fi
                shift
                ;;
        esac
    done
    
    # Check if database path is provided
    if [[ -z "$DB_PATH" ]]; then
        log_error "Database path is required!"
        echo
        show_help
        exit 1
    fi
}

# Initialize database
init_database() {
    local db_dir
    db_dir="$(dirname "$DB_PATH")"
    
    # Create parent directories if they don't exist
    if [[ ! -d "$db_dir" ]]; then
        log_info "Creating directory: $db_dir"
        mkdir -p "$db_dir"
    fi
    
    # Check if database exists
    if [[ -f "$DB_PATH" ]]; then
        if [[ $FORCE == true ]]; then
            log_warning "Removing existing database: $DB_PATH"
            rm -f "$DB_PATH"
        else
            log_error "Database already exists: $DB_PATH"
            log_error "Use --force to recreate the database (WARNING: this will delete all data)"
            exit 1
        fi
    fi
    
    # Validate SQL file exists
    if [[ ! -f "$SQL_FILE" ]]; then
        log_error "SQL initialization file not found: $SQL_FILE"
        exit 1
    fi
    
    log_info "Initializing database: $DB_PATH"
    log_info "Using SQL file: $SQL_FILE"
    
    # Initialize database with SQL script
    if [[ $VERBOSE == true ]]; then
        log_info "Executing SQL initialization script..."
        sqlite3 "$DB_PATH" < "$SQL_FILE"
    else
        sqlite3 "$DB_PATH" < "$SQL_FILE" 2>/dev/null
    fi
    
    # Verify database was created successfully
    if [[ ! -f "$DB_PATH" ]]; then
        log_error "Failed to create database!"
        exit 1
    fi
    
    # Check if tables were created
    local table_count
    table_count=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM sqlite_master WHERE type='table';")
    
    if [[ $table_count -eq 0 ]]; then
        log_error "No tables were created in the database!"
        exit 1
    fi
    
    log_success "Database initialized successfully!"
    log_info "Database path: $DB_PATH"
    log_info "Tables created: $table_count"
    
    if [[ $VERBOSE == true ]]; then
        log_info "Tables in database:"
        sqlite3 "$DB_PATH" "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;" | sed 's/^/  - /'
    fi
    
    # Show database info
    local db_size
    db_size=$(ls -lh "$DB_PATH" | awk '{print $5}')
    log_info "Database size: $db_size"
    
    # Test database connectivity
    local schema_version
    schema_version=$(sqlite3 "$DB_PATH" "SELECT value FROM system_config WHERE key='schema_version';" 2>/dev/null || echo "unknown")
    log_info "Schema version: $schema_version"
}

# Main function
main() {
    log_info "TagBox Database Initialization Script"
    log_info "======================================"
    
    parse_args "$@"
    check_sqlite3
    init_database
    
    echo
    log_success "Database initialization completed!"
    log_info "You can now use TagBox with this database:"
    log_info "  export DATABASE_URL=\"sqlite:$DB_PATH\""
    log_info "  cargo run --bin tagbox-cli -- --help"
}

# Run main function
main "$@"