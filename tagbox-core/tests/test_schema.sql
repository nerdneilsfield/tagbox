-- Test schema for unit tests
-- This file contains the minimum schema needed for running tests

-- Enable foreign keys
PRAGMA foreign_keys = ON;

-- Files table
CREATE TABLE IF NOT EXISTS files (
    id TEXT PRIMARY KEY,
    initial_hash TEXT,
    current_hash TEXT,
    relative_path TEXT,
    filename TEXT,
    title TEXT NOT NULL,
    year INTEGER,
    publisher TEXT,
    category_id TEXT,
    source_url TEXT,
    summaries TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    deleted_at TEXT,
    file_metadata TEXT,
    type_metadata TEXT,
    size INTEGER NOT NULL DEFAULT 0
);

-- System config table
CREATE TABLE IF NOT EXISTS system_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- File history table
CREATE TABLE IF NOT EXISTS file_history (
    id TEXT PRIMARY KEY,
    file_id TEXT NOT NULL,
    operation TEXT NOT NULL,
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

-- File access stats table
CREATE TABLE IF NOT EXISTS file_access_stats (
    file_id TEXT PRIMARY KEY,
    access_count INTEGER NOT NULL DEFAULT 0,
    last_accessed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_files_relative_path ON files(relative_path);
CREATE INDEX IF NOT EXISTS idx_files_current_hash ON files(current_hash);
CREATE INDEX IF NOT EXISTS idx_file_history_file_id ON file_history(file_id);
CREATE INDEX IF NOT EXISTS idx_file_history_changed_at ON file_history(changed_at);
CREATE INDEX IF NOT EXISTS idx_file_access_stats_access_count ON file_access_stats(access_count);