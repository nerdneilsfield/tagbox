use sqlx::sqlite::SqlitePool;
use std::env;
// use std::path::{Path, PathBuf}; // Removed unused imports

async fn execute_sql(db: &SqlitePool, sql: &str, description: &str) -> Result<(), sqlx::Error> {
    match sqlx::query(sql).execute(db).await {
        Ok(_) => {
            println!("Successfully executed: {}", description);
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to execute: {}\nError: {}\nSQL:\n{}", description, e, sql);
            Err(e)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量获取数据库URL
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    println!("Connecting to {}", database_url);
    
    // 连接数据库
    let db = SqlitePool::connect(&database_url).await?;
    println!("Successfully connected to database.");
    
    // 启用外键约束
    execute_sql(&db, "PRAGMA foreign_keys = ON;", "Enable foreign keys").await?;

    println!("Creating tables...");
    
    execute_sql(&db, "DROP TABLE IF EXISTS files;", "Drop files table (if exists)").await?;
    execute_sql(&db, "
        CREATE TABLE files (
            id TEXT PRIMARY KEY,
            initial_hash TEXT, -- 来自 database.md
            current_hash TEXT,
            relative_path TEXT, -- 来自 database.md
            filename TEXT,      -- 来自 database.md
            title TEXT NOT NULL,
            year INTEGER,       -- 来自 database.md
            publisher TEXT,     -- 来自 database.md
            category_id TEXT,   -- 来自 database.md
            source_url TEXT,    -- 来自 database.md
            summaries TEXT,     -- 来自 database.md (JSON)
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            is_deleted INTEGER NOT NULL DEFAULT 0, -- 对应 BOOLEAN
            deleted_at TEXT,     -- 来自 database.md
            FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL
        );
    ", "Create files table").await?;

    execute_sql(&db, "
        CREATE TABLE IF NOT EXISTS categories (
            id TEXT PRIMARY KEY,
            path TEXT NOT NULL UNIQUE,
            parent TEXT,
            updated_at TEXT NOT NULL
            -- FOREIGN KEY (parent) REFERENCES categories(id) ON DELETE SET NULL -- 根据设计文档，外键约束在最后统一添加，或由 ORM 处理
        );
    ", "Create categories table").await?;

    execute_sql(&db, "
        CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            path TEXT NOT NULL UNIQUE, -- 层级路径，如 技术/Rust
            parent_id TEXT,
            created_at TEXT NOT NULL,
            is_deleted INTEGER NOT NULL DEFAULT 0, -- 对应 BOOLEAN
            FOREIGN KEY (parent_id) REFERENCES tags(id) ON DELETE SET NULL
        );
    ", "Create tags table").await?;

    execute_sql(&db, "
        CREATE TABLE IF NOT EXISTS file_tags (
            file_id TEXT NOT NULL,
            tag_id TEXT NOT NULL,
            created_at TEXT NOT NULL,
            PRIMARY KEY (file_id, tag_id),
            FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );
    ", "Create file_tags table").await?;

    execute_sql(&db, "DROP TABLE IF EXISTS authors;", "Drop authors table (if exists)").await?;
    execute_sql(&db, "
        CREATE TABLE authors (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE, -- UNIQUE(name) 已在原表定义
            real_name TEXT,
            aliases TEXT, -- JSON 数组
            bio TEXT,
            homepage TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            is_deleted INTEGER NOT NULL DEFAULT 0 -- 对应 BOOLEAN
        );
    ", "Create authors table").await?;

    execute_sql(&db, "
        CREATE TABLE IF NOT EXISTS file_authors (
            file_id TEXT NOT NULL,
            author_id TEXT NOT NULL,
            PRIMARY KEY (file_id, author_id),
            FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
            FOREIGN KEY (author_id) REFERENCES authors(id) ON DELETE CASCADE
        );
    ", "Create file_authors table").await?;

    execute_sql(&db, "
        CREATE TABLE IF NOT EXISTS author_aliases (
            alias_id TEXT NOT NULL, -- 笔名作者 ID
            canonical_id TEXT NOT NULL, -- 主作者 ID
            note TEXT,
            merged_at TEXT,
            PRIMARY KEY (alias_id), -- 假设一个笔名只指向一个主名
            FOREIGN KEY (alias_id) REFERENCES authors(id) ON DELETE CASCADE,
            FOREIGN KEY (canonical_id) REFERENCES authors(id) ON DELETE CASCADE
        );
    ", "Create author_aliases table").await?;

    execute_sql(&db, "
        CREATE TABLE IF NOT EXISTS file_links (
            source_id TEXT NOT NULL, -- 来源文件 ID
            target_id TEXT NOT NULL, -- 目标文件 ID
            relation TEXT,
            comment TEXT,
            created_at TEXT NOT NULL,
            PRIMARY KEY (source_id, target_id),
            FOREIGN KEY (source_id) REFERENCES files(id) ON DELETE CASCADE,
            FOREIGN KEY (target_id) REFERENCES files(id) ON DELETE CASCADE
        );
    ", "Create file_links table").await?;

    execute_sql(&db, "
        CREATE TABLE IF NOT EXISTS file_metadata (
            file_id TEXT NOT NULL,
            key TEXT NOT NULL,
            value TEXT,
            PRIMARY KEY (file_id, key),
            FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
        );
    ", "Create file_metadata table").await?;

    execute_sql(&db, "
        CREATE VIRTUAL TABLE IF NOT EXISTS files_fts USING fts5(
            title, 
            tags,
            summaries, 
            authors,
            content=\'files\', 
            content_rowid=\'id\'
        );
    ", "Create files_fts FTS5 table").await?;

    execute_sql(&db, "
        CREATE TABLE IF NOT EXISTS file_access_log (
            file_id TEXT NOT NULL,
            accessed_at TEXT NOT NULL,
            method TEXT,
            FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
        );
    ", "Create file_access_log table").await?;

    println!("Creating indexes...");
    execute_sql(&db, "CREATE INDEX IF NOT EXISTS idx_files_category ON files(category_id);", "Create index idx_files_category").await?;
    execute_sql(&db, "CREATE INDEX IF NOT EXISTS idx_files_year ON files(year);", "Create index idx_files_year").await?;
    execute_sql(&db, "CREATE INDEX IF NOT EXISTS idx_files_current_hash ON files(current_hash);", "Create index idx_files_current_hash").await?;
    execute_sql(&db, "CREATE INDEX IF NOT EXISTS idx_tags_path ON tags(path);", "Create index idx_tags_path").await?;
    execute_sql(&db, "CREATE INDEX IF NOT EXISTS idx_authors_name ON authors(name);", "Create index idx_authors_name").await?;
    println!("Indexes created successfully.");

    println!("Database initialized successfully!");
    Ok(())
}