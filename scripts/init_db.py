#!/usr/bin/env python3
"""
TagBox Database Initialization Script (Python)
This script initializes a TagBox database using the SQL schema
"""

import argparse
import os
import sqlite3
import sys
from pathlib import Path
from typing import Optional

# Colors for terminal output
class Colors:
    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    NC = '\033[0m'  # No Color

def log_info(message: str) -> None:
    """Log info message"""
    print(f"{Colors.BLUE}[INFO]{Colors.NC} {message}")

def log_success(message: str) -> None:
    """Log success message"""
    print(f"{Colors.GREEN}[SUCCESS]{Colors.NC} {message}")

def log_warning(message: str) -> None:
    """Log warning message"""
    print(f"{Colors.YELLOW}[WARNING]{Colors.NC} {message}")

def log_error(message: str) -> None:
    """Log error message"""
    print(f"{Colors.RED}[ERROR]{Colors.NC} {message}", file=sys.stderr)

def check_sqlite3() -> bool:
    """Check if sqlite3 module is available and working"""
    try:
        # Test sqlite3 connection
        conn = sqlite3.connect(':memory:')
        cursor = conn.cursor()
        cursor.execute('SELECT sqlite_version();')
        version = cursor.fetchone()[0]
        conn.close()
        
        log_info(f"Using SQLite version: {version}")
        return True
    except Exception as e:
        log_error(f"SQLite3 not available: {e}")
        log_error("Please ensure Python was compiled with SQLite3 support")
        return False

def read_sql_file(sql_file_path: Path) -> str:
    """Read SQL initialization file"""
    try:
        with open(sql_file_path, 'r', encoding='utf-8') as f:
            return f.read()
    except FileNotFoundError:
        log_error(f"SQL initialization file not found: {sql_file_path}")
        sys.exit(1)
    except Exception as e:
        log_error(f"Failed to read SQL file: {e}")
        sys.exit(1)

def execute_sql_script(db_path: Path, sql_content: str, verbose: bool = False) -> None:
    """Execute SQL script on database"""
    try:
        conn = sqlite3.connect(str(db_path))
        
        if verbose:
            log_info("Executing SQL initialization script...")
        
        # Enable foreign keys
        conn.execute('PRAGMA foreign_keys = ON;')
        
        # Execute the SQL script
        conn.executescript(sql_content)
        conn.commit()
        conn.close()
        
    except sqlite3.Error as e:
        log_error(f"SQLite error: {e}")
        sys.exit(1)
    except Exception as e:
        log_error(f"Failed to execute SQL script: {e}")
        sys.exit(1)

def verify_database(db_path: Path, verbose: bool = False) -> None:
    """Verify database was created successfully"""
    try:
        conn = sqlite3.connect(str(db_path))
        cursor = conn.cursor()
        
        # Check if tables were created
        cursor.execute("SELECT COUNT(*) FROM sqlite_master WHERE type='table';")
        table_count = cursor.fetchone()[0]
        
        if table_count == 0:
            log_error("No tables were created in the database!")
            sys.exit(1)
        
        log_success("Database initialized successfully!")
        log_info(f"Database path: {db_path}")
        log_info(f"Tables created: {table_count}")
        
        if verbose:
            log_info("Tables in database:")
            cursor.execute("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;")
            tables = cursor.fetchall()
            for table in tables:
                print(f"  - {table[0]}")
        
        # Show database info
        db_size = db_path.stat().st_size
        if db_size < 1024:
            size_str = f"{db_size} bytes"
        elif db_size < 1024 * 1024:
            size_str = f"{db_size / 1024:.1f} KB"
        else:
            size_str = f"{db_size / (1024 * 1024):.1f} MB"
        
        log_info(f"Database size: {size_str}")
        
        # Test database connectivity and get schema version
        try:
            cursor.execute("SELECT value FROM system_config WHERE key='schema_version';")
            result = cursor.fetchone()
            schema_version = result[0] if result else "unknown"
            log_info(f"Schema version: {schema_version}")
        except sqlite3.Error:
            log_info("Schema version: unknown")
        
        conn.close()
        
    except Exception as e:
        log_error(f"Failed to verify database: {e}")
        sys.exit(1)

def initialize_database(db_path: Path, sql_file_path: Path, force: bool = False, verbose: bool = False) -> None:
    """Initialize TagBox database"""
    # Create parent directories if they don't exist
    db_path.parent.mkdir(parents=True, exist_ok=True)
    
    # Check if database exists
    if db_path.exists():
        if force:
            log_warning(f"Removing existing database: {db_path}")
            db_path.unlink()
        else:
            log_error(f"Database already exists: {db_path}")
            log_error("Use --force to recreate the database (WARNING: this will delete all data)")
            sys.exit(1)
    
    # Validate SQL file exists
    if not sql_file_path.exists():
        log_error(f"SQL initialization file not found: {sql_file_path}")
        sys.exit(1)
    
    log_info(f"Initializing database: {db_path}")
    log_info(f"Using SQL file: {sql_file_path}")
    
    # Read and execute SQL script
    sql_content = read_sql_file(sql_file_path)
    execute_sql_script(db_path, sql_content, verbose)
    
    # Verify database
    verify_database(db_path, verbose)

def main() -> None:
    """Main function"""
    parser = argparse.ArgumentParser(
        description="TagBox Database Initialization Script",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Initialize a new database
  python init_db.py ./tagbox.db
  
  # Initialize with verbose output
  python init_db.py -v ./data/tagbox.db
  
  # Force recreate existing database
  python init_db.py --force ./tagbox.db
  
Notes:
  - The script uses Python's built-in sqlite3 module
  - Database file and parent directories will be created if they don't exist
  - Use --force carefully as it will delete all existing data
        """
    )
    
    parser.add_argument(
        'database_path',
        type=Path,
        help='Path to the SQLite database file (will be created if doesn\'t exist)'
    )
    
    parser.add_argument(
        '-f', '--force',
        action='store_true',
        help='Force recreate database (will delete existing data!)'
    )
    
    parser.add_argument(
        '-v', '--verbose',
        action='store_true',
        help='Enable verbose output'
    )
    
    args = parser.parse_args()
    
    # Get script directory and SQL file path
    script_dir = Path(__file__).parent
    sql_file_path = script_dir / "init-database.sql"
    
    log_info("TagBox Database Initialization Script")
    log_info("======================================")
    
    # Check sqlite3 availability
    if not check_sqlite3():
        sys.exit(1)
    
    # Initialize database
    initialize_database(
        db_path=args.database_path,
        sql_file_path=sql_file_path,
        force=args.force,
        verbose=args.verbose
    )
    
    print()
    log_success("Database initialization completed!")
    log_info("You can now use TagBox with this database:")
    log_info(f"  export DATABASE_URL=\"sqlite:{args.database_path}\"")
    log_info("  cargo run --bin tagbox-cli -- --help")

if __name__ == "__main__":
    main()