# TagBox Database Initialization Scripts

This directory contains scripts to initialize TagBox databases using different tools and platforms.

## Available Scripts

### 1. SQL Script (`init-database.sql`)
Pure SQL script that can be used with any SQLite3 client.

```bash
# Using sqlite3 command line
sqlite3 tagbox.db < scripts/init-database.sql

# Using any SQLite GUI tool
# Import/execute the init-database.sql file
```

### 2. Bash Script (`init-db.sh`)
Cross-platform shell script for Unix-like systems (Linux, macOS, WSL).

```bash
# Make executable (if needed)
chmod +x scripts/init-db.sh

# Initialize new database
./scripts/init-db.sh ./tagbox.db

# With verbose output
./scripts/init-db.sh -v ./data/tagbox.db

# Force recreate existing database
./scripts/init-db.sh --force ./tagbox.db

# Show help
./scripts/init-db.sh --help
```

### 3. PowerShell Script (`init-db.ps1`)
Windows PowerShell script.

```powershell
# Initialize new database
.\scripts\init-db.ps1 .\tagbox.db

# With verbose output
.\scripts\init-db.ps1 -Verbose .\data\tagbox.db

# Force recreate existing database
.\scripts\init-db.ps1 -Force .\tagbox.db

# Show help
.\scripts\init-db.ps1 -Help
```

### 4. Python Script (`init_db.py`)
Cross-platform Python script (requires Python 3.6+).

```bash
# Make executable (Unix-like systems)
chmod +x scripts/init_db.py

# Initialize new database
python3 scripts/init_db.py ./tagbox.db

# With verbose output
python3 scripts/init_db.py -v ./data/tagbox.db

# Force recreate existing database
python3 scripts/init_db.py --force ./tagbox.db

# Show help
python3 scripts/init_db.py --help
```

### 5. Rust Binary (`tagbox-init-db`)
Built-in Rust tool (requires compilation).

```bash
# Build the tool
cargo build --bin tagbox-init-db

# Set DATABASE_URL environment variable
export DATABASE_URL="sqlite:./tagbox.db"

# Initialize database
cargo run --bin tagbox-init-db
```

## Requirements

### SQL Script
- SQLite3 command-line tool or any SQLite client

### Bash Script
- bash shell
- sqlite3 command-line tool

### PowerShell Script
- PowerShell 5.0+ or PowerShell Core
- sqlite3.exe in PATH

### Python Script
- Python 3.6+
- No additional dependencies (uses built-in sqlite3 module)

### Rust Binary
- Rust toolchain
- All project dependencies

## Installation of SQLite3

### macOS
```bash
# Using Homebrew
brew install sqlite3

# Using MacPorts
sudo port install sqlite3
```

### Ubuntu/Debian
```bash
sudo apt update
sudo apt install sqlite3
```

### CentOS/RHEL/Fedora
```bash
# CentOS/RHEL
sudo yum install sqlite

# Fedora
sudo dnf install sqlite
```

### Windows
```powershell
# Using Chocolatey
choco install sqlite

# Using Scoop
scoop install sqlite

# Or download from https://www.sqlite.org/download.html
```

## Database Schema

The scripts create the following tables:

### Core Tables
- `files` - Main file metadata
- `authors` - Author information with aliases support
- `tags` - Hierarchical tag system
- `categories` - File categories

### Relationship Tables
- `file_authors` - File-author many-to-many relationships
- `file_tags` - File-tag many-to-many relationships
- `author_aliases` - Author name normalization
- `file_links` - File-to-file references
- `file_metadata` - Additional key-value metadata

### System Tables
- `system_config` - System configuration
- `file_history` - Operation history
- `file_access_stats` - Access statistics
- `file_access_log` - Access logging

### Search Tables
- `files_fts` - Full-text search (FTS5) virtual table

## Usage Examples

### Basic Usage
```bash
# Initialize database
./scripts/init-db.sh ./my-library.db

# Set environment variable
export DATABASE_URL="sqlite:./my-library.db"

# Use with TagBox CLI
cargo run --bin tagbox-cli -- search "rust programming"
```

### Development Setup
```bash
# Create development database
python3 scripts/init_db.py -v ./dev.db

# Run tests with new database
export DATABASE_URL="sqlite:./dev.db"
cargo test
```

### Production Setup
```bash
# Create production database with proper path
sudo mkdir -p /var/lib/tagbox
sudo ./scripts/init-db.sh /var/lib/tagbox/library.db
sudo chown tagbox:tagbox /var/lib/tagbox/library.db
```

## Troubleshooting

### Permission Errors
```bash
# Make scripts executable
chmod +x scripts/init-db.sh scripts/init_db.py

# Check file permissions
ls -la scripts/
```

### SQLite3 Not Found
```bash
# Check if sqlite3 is installed
which sqlite3
sqlite3 --version

# Install if missing (see installation section above)
```

### Database Already Exists
```bash
# Use --force flag to recreate
./scripts/init-db.sh --force ./tagbox.db

# Or manually remove
rm -f ./tagbox.db
```

### Python Import Errors
```bash
# Check Python version
python3 --version

# Test sqlite3 module
python3 -c "import sqlite3; print(sqlite3.sqlite_version)"
```

## Validation

After initialization, you can validate the database:

```bash
# Check tables
sqlite3 tagbox.db ".tables"

# Check schema version
sqlite3 tagbox.db "SELECT * FROM system_config WHERE key='schema_version';"

# Check table counts
sqlite3 tagbox.db "SELECT COUNT(*) FROM sqlite_master WHERE type='table';"
```

## Notes

- All scripts create identical database schemas
- The database uses WAL mode for better concurrent access
- Foreign key constraints are enabled
- Default categories and tags are pre-populated
- System configuration tracks initialization metadata