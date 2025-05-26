use crate::errors::{Result, TagboxError};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::Path;
use std::fs;
use tracing::{info, warn};
// use libsqlite3_sys::{sqlite3, sqlite3_db_handle};
// use signal_tokenizer::{CJKTokenizer, register_tokenizer};

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// 初始化数据库连接池
    pub async fn new(path: &Path) -> Result<Self> {
        // 确保父目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| TagboxError::Io(e))?;
        }
        
        let url = format!("sqlite:{}", path.to_string_lossy());
        
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
            .map_err(|e| TagboxError::Database(e))?;
        
        // 启用外键约束
        sqlx::query("PRAGMA foreign_keys = ON;")
            .execute(&pool)
            .await
            .map_err(|e| TagboxError::Database(e))?;
            
        // 启用WAL日志模式提高并发性能
        sqlx::query("PRAGMA journal_mode = WAL;")
            .execute(&pool)
            .await
            .map_err(|e| TagboxError::Database(e))?;
        
        // 注册 Signal-FTS5 扩展
        // register_signal_fts5_extension(&pool).await?;
            
        Ok(Database { pool })
    }
    
    /// 应用迁移脚本创建表结构
    pub async fn migrate(&self) -> Result<()> {
        info!("开始应用数据库迁移...");
        
        // 创建文件表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS files (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                original_filename TEXT NOT NULL,
                hash TEXT NOT NULL UNIQUE,
                current_hash TEXT,
                path TEXT NOT NULL,
                original_path TEXT,
                category1 TEXT NOT NULL,
                category2 TEXT,
                category3 TEXT,
                summary TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                last_accessed TEXT,
                is_deleted INTEGER NOT NULL DEFAULT 0,
                UNIQUE(hash)
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| TagboxError::Database(e))?;
        
        // 创建作者表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS authors (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(name)
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| TagboxError::Database(e))?;
        
        // 创建文件-作者关联表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS file_authors (
                file_id TEXT NOT NULL,
                author_id TEXT NOT NULL,
                PRIMARY KEY (file_id, author_id),
                FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
                FOREIGN KEY (author_id) REFERENCES authors(id) ON DELETE CASCADE
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| TagboxError::Database(e))?;
        
        // 创建标签表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                parent_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(name),
                FOREIGN KEY (parent_id) REFERENCES tags(id) ON DELETE SET NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| TagboxError::Database(e))?;
        
        // 创建文件-标签关联表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS file_tags (
                file_id TEXT NOT NULL,
                tag_id TEXT NOT NULL,
                PRIMARY KEY (file_id, tag_id),
                FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| TagboxError::Database(e))?;
        
        // 创建作者别名表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS author_aliases (
                alias TEXT NOT NULL,
                author_id TEXT NOT NULL,
                PRIMARY KEY (alias),
                FOREIGN KEY (author_id) REFERENCES authors(id) ON DELETE CASCADE
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| TagboxError::Database(e))?;
        
        // 创建文件元数据表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS file_metadata (
                file_id TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                PRIMARY KEY (file_id, key),
                FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| TagboxError::Database(e))?;
        
        // 创建文件关联表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS file_links (
                file_id_a TEXT NOT NULL,
                file_id_b TEXT NOT NULL,
                relation_type TEXT NOT NULL,
                created_at TEXT NOT NULL,
                PRIMARY KEY (file_id_a, file_id_b),
                FOREIGN KEY (file_id_a) REFERENCES files(id) ON DELETE CASCADE,
                FOREIGN KEY (file_id_b) REFERENCES files(id) ON DELETE CASCADE
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| TagboxError::Database(e))?;
        
        // 创建全文搜索虚拟表 (使用 Signal CJK 分词器)
        let create_fts_result = sqlx::query(
            r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS files_fts USING fts5(
                title, 
                authors,
                summary,
                tags,
                content='files',
                content_rowid='rowid',
                tokenize='signal_cjk porter unicode61 remove_diacritics 1'
            );
            "#,
        )
        .execute(&self.pool)
        .await;
        
        match create_fts_result {
            Ok(_) => info!("FTS5虚拟表创建成功，使用 Signal CJK 分词器"),
            Err(e) => {
                warn!("无法创建带Signal CJK分词器的FTS5表，尝试使用标准分词器: {}", e);
                
                // 尝试使用标准分词器
                let create_fts5_standard = sqlx::query(
                    r#"
                    CREATE VIRTUAL TABLE IF NOT EXISTS files_fts USING fts5(
                        title, 
                        authors,
                        summary,
                        tags,
                        content='files',
                        content_rowid='rowid',
                        tokenize='unicode61 remove_diacritics 1'
                    );
                    "#,
                )
                .execute(&self.pool)
                .await;
                
                match create_fts5_standard {
                    Ok(_) => info!("FTS5虚拟表创建成功，使用标准分词器"),
                    Err(e2) => {
                        warn!("无法创建标准FTS5表，尝试使用FTS4: {}", e2);
                        
                        // 尝试创建基本的FTS4表（更广泛支持）
                        let create_fts4_result = sqlx::query(
                            r#"
                            CREATE VIRTUAL TABLE IF NOT EXISTS files_fts USING fts4(
                                title, 
                                authors,
                                summary,
                                tags,
                                content='files',
                                tokenize=simple
                            );
                            "#,
                        )
                        .execute(&self.pool)
                        .await
                        .map_err(|e| TagboxError::Database(e))?;
                        
                        info!("FTS4虚拟表创建成功（作为备选）");
                    }
                }
            }
        }
        
        // 创建触发器在文件更新时更新FTS索引
        sqlx::query(
            r#"
            CREATE TRIGGER IF NOT EXISTS files_ai AFTER INSERT ON files BEGIN
                INSERT INTO files_fts(rowid, title, authors, summary, tags)
                VALUES (new.rowid, new.title, '', new.summary, '');
            END;
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| TagboxError::Database(e))?;
        
        sqlx::query(
            r#"
            CREATE TRIGGER IF NOT EXISTS files_ad AFTER DELETE ON files BEGIN
                DELETE FROM files_fts WHERE rowid = old.rowid;
            END;
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| TagboxError::Database(e))?;
        
        sqlx::query(
            r#"
            CREATE TRIGGER IF NOT EXISTS files_au AFTER UPDATE ON files BEGIN
                DELETE FROM files_fts WHERE rowid = old.rowid;
                INSERT INTO files_fts(rowid, title, authors, summary, tags)
                VALUES (new.rowid, new.title, '', new.summary, '');
            END;
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| TagboxError::Database(e))?;
        
        info!("数据库迁移完成");
        Ok(())
    }
    
    /// 获取数据库连接池引用
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

/*
/// 注册 Signal FTS5 扩展的辅助函数
async fn register_signal_fts5_extension(pool: &SqlitePool) -> Result<()> {
    // 获取 SQLite 数据库句柄
    let mut conn = pool.acquire().await.map_err(|e| TagboxError::Database(e))?;
    
    // 获取 SQLite3 原生句柄
    let db_ptr = unsafe {
        let conn_ptr = conn.as_raw(); // 获取 SQLite 连接指针
        sqlite3_db_handle(conn_ptr as *mut _) // 获取底层 sqlite3 句柄
    };
    
    if db_ptr.is_null() {
        return Err(TagboxError::Config("无法获取SQLite数据库句柄".to_string()));
    }
    
    // 注册 Signal CJK 分词器
    let tokenizer = CJKTokenizer::new();
    
    unsafe {
        match register_tokenizer(db_ptr, "signal_cjk", tokenizer) {
            Ok(_) => {
                info!("Signal CJK 分词器注册成功");
                Ok(())
            },
            Err(err) => {
                warn!("无法注册 Signal CJK 分词器: {:?}", err);
                Err(TagboxError::Config(format!("Signal CJK 分词器注册失败: {:?}", err)))
            }
        }
    }
}
*/