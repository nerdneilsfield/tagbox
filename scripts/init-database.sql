-- TagBox Database Initialization Script
-- This script creates all required tables and indexes for TagBox

-- Enable foreign key constraints
PRAGMA foreign_keys = ON;

-- Enable WAL mode for better performance
PRAGMA journal_mode = WAL;

-- ========================================
-- Core Tables
-- ========================================

-- Files table - stores file metadata with simplified three-level categories
DROP TABLE IF EXISTS files;
CREATE TABLE files (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    initial_hash TEXT NOT NULL UNIQUE,
    current_hash TEXT,
    relative_path TEXT NOT NULL,
    filename TEXT NOT NULL,
    
    -- Basic metadata
    year INTEGER,
    publisher TEXT,
    source_url TEXT,
    summary TEXT,
    
    -- Simplified three-level categories (supports "cat1/cat2/cat3" format)
    category1 TEXT,
    category2 TEXT,
    category3 TEXT,
    
    -- Full text content (for search)
    full_text TEXT,
    
    -- System fields
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    deleted_at TEXT,
    file_metadata TEXT,  -- JSON format
    type_metadata TEXT,  -- JSON format
    
    UNIQUE(initial_hash)
);

-- Note: Categories table removed - now using simplified category1/category2/category3 fields

-- Tags table with hierarchical support
DROP TABLE IF EXISTS tags;
CREATE TABLE tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL UNIQUE,  -- hierarchical path like "技术/Rust"
    parent_id TEXT,
    created_at TEXT NOT NULL,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (parent_id) REFERENCES tags(id) ON DELETE SET NULL
);

-- Authors table with extended information
DROP TABLE IF EXISTS authors;
CREATE TABLE authors (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    real_name TEXT,
    aliases TEXT,  -- JSON array
    bio TEXT,
    homepage TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    is_deleted INTEGER NOT NULL DEFAULT 0
);

-- ========================================
-- Relationship Tables
-- ========================================

-- File-Author relationships (many-to-many)
CREATE TABLE IF NOT EXISTS file_authors (
    file_id TEXT NOT NULL,
    author_id TEXT NOT NULL,
    PRIMARY KEY (file_id, author_id),
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
    FOREIGN KEY (author_id) REFERENCES authors(id) ON DELETE CASCADE
);

-- File-Tag relationships (many-to-many)
CREATE TABLE IF NOT EXISTS file_tags (
    file_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (file_id, tag_id),
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- Author aliases for name normalization
CREATE TABLE IF NOT EXISTS author_aliases (
    alias_id TEXT NOT NULL,  -- alias author ID
    canonical_id TEXT NOT NULL,  -- canonical author ID
    note TEXT,
    merged_at TEXT,
    PRIMARY KEY (alias_id),
    FOREIGN KEY (alias_id) REFERENCES authors(id) ON DELETE CASCADE,
    FOREIGN KEY (canonical_id) REFERENCES authors(id) ON DELETE CASCADE
);

-- File links (references between files)
CREATE TABLE IF NOT EXISTS file_links (
    source_id TEXT NOT NULL,  -- source file ID
    target_id TEXT NOT NULL,  -- target file ID
    relation TEXT,
    comment TEXT,
    created_at TEXT NOT NULL,
    PRIMARY KEY (source_id, target_id),
    FOREIGN KEY (source_id) REFERENCES files(id) ON DELETE CASCADE,
    FOREIGN KEY (target_id) REFERENCES files(id) ON DELETE CASCADE
);

-- File metadata key-value store (optional, prefer JSON fields)
CREATE TABLE IF NOT EXISTS file_metadata (
    file_id TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT,
    PRIMARY KEY (file_id, key),
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
);

-- ========================================
-- Full-Text Search
-- ========================================

-- FTS5 virtual table for full-text search (includes full_text field)
CREATE VIRTUAL TABLE IF NOT EXISTS files_fts USING fts5(
    title, 
    authors,
    summary, 
    tags,
    full_text,
    content='files', 
    content_rowid='rowid'
);

-- ========================================
-- System Tables
-- ========================================

-- File access log
CREATE TABLE IF NOT EXISTS file_access_log (
    file_id TEXT NOT NULL,
    accessed_at TEXT NOT NULL,
    method TEXT,  -- CLI, GUI, API
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
);

-- System configuration
CREATE TABLE IF NOT EXISTS system_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- File operation history
CREATE TABLE IF NOT EXISTS file_history (
    id TEXT PRIMARY KEY,
    file_id TEXT NOT NULL,
    operation TEXT NOT NULL,  -- create, update, move, delete, access
    old_hash TEXT,
    new_hash TEXT,
    old_path TEXT,
    new_path TEXT,
    old_size INTEGER,
    new_size INTEGER,
    changed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    changed_by TEXT,
    reason TEXT,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
);

-- File access statistics
CREATE TABLE IF NOT EXISTS file_access_stats (
    file_id TEXT PRIMARY KEY,
    access_count INTEGER NOT NULL DEFAULT 0,
    last_accessed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
);

-- ========================================
-- Indexes for Performance
-- ========================================

-- Primary indexes
CREATE INDEX IF NOT EXISTS idx_files_category1 ON files(category1);
CREATE INDEX IF NOT EXISTS idx_files_category2 ON files(category2);
CREATE INDEX IF NOT EXISTS idx_files_category3 ON files(category3);
CREATE INDEX IF NOT EXISTS idx_files_year ON files(year);
CREATE INDEX IF NOT EXISTS idx_files_current_hash ON files(current_hash);
CREATE INDEX IF NOT EXISTS idx_files_initial_hash ON files(initial_hash);
CREATE INDEX IF NOT EXISTS idx_files_title ON files(title);
CREATE INDEX IF NOT EXISTS idx_files_created_at ON files(created_at);
CREATE INDEX IF NOT EXISTS idx_files_updated_at ON files(updated_at);

-- Tag indexes
CREATE INDEX IF NOT EXISTS idx_tags_path ON tags(path);
CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name);
CREATE INDEX IF NOT EXISTS idx_tags_parent_id ON tags(parent_id);

-- Author indexes
CREATE INDEX IF NOT EXISTS idx_authors_name ON authors(name);
CREATE INDEX IF NOT EXISTS idx_authors_real_name ON authors(real_name);

-- Relationship indexes
CREATE INDEX IF NOT EXISTS idx_file_authors_file_id ON file_authors(file_id);
CREATE INDEX IF NOT EXISTS idx_file_authors_author_id ON file_authors(author_id);
CREATE INDEX IF NOT EXISTS idx_file_tags_file_id ON file_tags(file_id);
CREATE INDEX IF NOT EXISTS idx_file_tags_tag_id ON file_tags(tag_id);

-- System table indexes
CREATE INDEX IF NOT EXISTS idx_file_history_file_id ON file_history(file_id);
CREATE INDEX IF NOT EXISTS idx_file_history_changed_at ON file_history(changed_at);
CREATE INDEX IF NOT EXISTS idx_file_history_operation ON file_history(operation);
CREATE INDEX IF NOT EXISTS idx_file_access_stats_access_count ON file_access_stats(access_count);
CREATE INDEX IF NOT EXISTS idx_file_access_log_file_id ON file_access_log(file_id);
CREATE INDEX IF NOT EXISTS idx_file_access_log_accessed_at ON file_access_log(accessed_at);

-- JSON indexes (SQLite 3.38.0+)
CREATE INDEX IF NOT EXISTS idx_files_file_metadata ON files(file_metadata);
CREATE INDEX IF NOT EXISTS idx_files_type_metadata ON files(type_metadata);

-- ========================================
-- Insert Initial Data
-- ========================================

-- Insert default system configuration
INSERT OR IGNORE INTO system_config (key, value, description) VALUES
    ('schema_version', '1.0.0', 'Database schema version'),
    ('created_at', datetime('now'), 'Database creation timestamp'),
    ('last_migration', datetime('now'), 'Last migration timestamp');

-- Note: Default categories removed - now using simplified string-based categories

-- Insert default tags
INSERT OR IGNORE INTO tags (id, name, path, parent_id, created_at, is_deleted) VALUES
    ('tag_misc', 'misc', 'misc', NULL, datetime('now'), 0),
    ('tag_tech', '技术', '技术', NULL, datetime('now'), 0),
    ('tag_literature', '文学', '文学', NULL, datetime('now'), 0);

-- ========================================
-- Database Info
-- ========================================

-- Log database initialization
INSERT OR REPLACE INTO system_config (key, value, description) VALUES
    ('initialized_at', datetime('now'), 'Database initialization timestamp'),
    ('initialization_method', 'sql_script', 'How the database was initialized');