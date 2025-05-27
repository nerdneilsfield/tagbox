use crate::errors::{Result, TagboxError};
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};
use std::fs;
use std::path::Path;
use tracing::{info, warn};

pub struct Database {
    pool: SqlitePool,
}

// Signal FTS5 extension entry point
extern "C" {
    fn signal_fts5_tokenizer_init(
        db: *mut std::ffi::c_void,
        pz_err_msg: *mut *mut std::os::raw::c_char,
        p_api: *const std::ffi::c_void,
    ) -> std::os::raw::c_int;
}

// 添加sqlite3_auto_extension函数绑定
extern "C" {
    fn sqlite3_auto_extension(
        xEntryPoint: unsafe extern "C" fn(
            db: *mut std::ffi::c_void,
            pz_err_msg: *mut *mut std::os::raw::c_char,
            p_api: *const std::ffi::c_void,
        ) -> std::os::raw::c_int,
    ) -> std::os::raw::c_int;
}

impl Database {
    /// 初始化数据库连接池
    pub async fn new(path: &Path) -> Result<Self> {
        // 确保父目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(TagboxError::Io)?;
        }

        // 在创建任何连接之前注册Signal tokenizer为自动扩展
        register_signal_auto_extension();

        let url = format!("sqlite:{}", path.to_string_lossy());

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    // 启用FTS5扩展并测试Signal tokenizer
                    enable_fts5_and_signal_tokenizer(conn).await
                })
            })
            .connect(&url)
            .await
            .map_err(TagboxError::Database)?;

        // 启用外键约束
        sqlx::query("PRAGMA foreign_keys = ON;")
            .execute(&pool)
            .await
            .map_err(TagboxError::Database)?;

        // 启用WAL日志模式提高并发性能
        sqlx::query("PRAGMA journal_mode = WAL;")
            .execute(&pool)
            .await
            .map_err(TagboxError::Database)?;

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
                initial_hash TEXT NOT NULL UNIQUE,
                current_hash TEXT,
                relative_path TEXT NOT NULL,
                filename TEXT NOT NULL,
                year INTEGER,
                publisher TEXT,
                category_id TEXT,
                source_url TEXT,
                summary TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                is_deleted INTEGER NOT NULL DEFAULT 0,
                deleted_at TEXT,
                file_metadata TEXT,
                type_metadata TEXT,
                UNIQUE(initial_hash)
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(TagboxError::Database)?;

        // 创建作者表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS authors (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                real_name TEXT,
                aliases TEXT,
                bio TEXT,
                homepage TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                is_deleted INTEGER NOT NULL DEFAULT 0
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(TagboxError::Database)?;

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
        .map_err(TagboxError::Database)?;

        // 创建标签表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE,
                parent_id TEXT,
                created_at TEXT NOT NULL,
                is_deleted INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (parent_id) REFERENCES tags(id) ON DELETE SET NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(TagboxError::Database)?;

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
        .map_err(TagboxError::Database)?;

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
        .map_err(TagboxError::Database)?;

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
        .map_err(TagboxError::Database)?;

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
        .map_err(TagboxError::Database)?;

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
                tokenize='signal_tokenizer unicode61 remove_diacritics 1'
            );
            "#,
        )
        .execute(&self.pool)
        .await;

        match create_fts_result {
            Ok(_) => info!("FTS5虚拟表创建成功，使用 Signal Tokenizer 分词器"),
            Err(e) => {
                warn!(
                    "无法创建带Signal Tokenizer分词器的FTS5表，尝试使用标准分词器: {}",
                    e
                );

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
                        let _create_fts4_result = sqlx::query(
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
                        .map_err(TagboxError::Database)?;

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
        .map_err(TagboxError::Database)?;

        sqlx::query(
            r#"
            CREATE TRIGGER IF NOT EXISTS files_ad AFTER DELETE ON files BEGIN
                DELETE FROM files_fts WHERE rowid = old.rowid;
            END;
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(TagboxError::Database)?;

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
        .map_err(TagboxError::Database)?;

        // 创建系统配置表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS system_config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(TagboxError::Database)?;

        // 创建文件历史表
        sqlx::query(
            r#"
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
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(TagboxError::Database)?;

        // 创建文件访问统计表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS file_access_stats (
                file_id TEXT NOT NULL,
                access_date DATE NOT NULL,
                access_type TEXT NOT NULL,
                access_count INTEGER DEFAULT 1,
                last_accessed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (file_id, access_date, access_type),
                FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(TagboxError::Database)?;

        // 创建索引
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_files_initial_hash ON files(initial_hash);")
            .execute(&self.pool)
            .await
            .map_err(TagboxError::Database)?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_files_current_hash ON files(current_hash);")
            .execute(&self.pool)
            .await
            .map_err(TagboxError::Database)?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_file_history_file_id ON file_history(file_id);",
        )
        .execute(&self.pool)
        .await
        .map_err(TagboxError::Database)?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_file_history_changed_at ON file_history(changed_at);",
        )
        .execute(&self.pool)
        .await
        .map_err(TagboxError::Database)?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_file_access_stats_access_date ON file_access_stats(access_date);")
            .execute(&self.pool)
            .await
            .map_err(TagboxError::Database)?;

        info!("数据库迁移完成");
        Ok(())
    }

    /// 获取数据库连接池引用
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

/// 启用FTS5并测试Signal tokenizer
async fn enable_fts5_and_signal_tokenizer(conn: &mut sqlx::SqliteConnection) -> sqlx::Result<()> {
    // 首先检查SQLite编译选项
    check_sqlite_compile_options(conn).await;

    // 直接测试FTS5表创建而不依赖fts5_version()函数
    test_fts5_table_creation(conn).await;

    Ok(())
}

/// 测试FTS5表创建
async fn test_fts5_table_creation(conn: &mut sqlx::SqliteConnection) {
    // 尝试创建一个简单的FTS5表来测试
    let test_result =
        sqlx::query("CREATE VIRTUAL TABLE IF NOT EXISTS test_fts5_basic USING fts5(content)")
            .execute(&mut *conn)
            .await;

    match test_result {
        Ok(_) => {
            tracing::debug!("✅ FTS5 is working! Can create FTS5 tables");

            // 清理测试表
            let _ = sqlx::query("DROP TABLE IF EXISTS test_fts5_basic")
                .execute(&mut *conn)
                .await;

            // 测试Signal tokenizer是否可用
            test_signal_tokenizer_availability(conn).await;
        }
        Err(e) => {
            tracing::warn!("❌ FTS5 table creation failed: {:?}", e);
            tracing::debug!("Will use fallback search without FTS5");
        }
    }
}

/// 检查SQLite编译选项
async fn check_sqlite_compile_options(conn: &mut sqlx::SqliteConnection) {
    match sqlx::query("PRAGMA compile_options")
        .fetch_all(&mut *conn)
        .await
    {
        Ok(rows) => {
            tracing::debug!("SQLite compile options:");
            let mut has_fts5 = false;
            for row in rows {
                if let Ok(option) = row.try_get::<String, _>(0) {
                    tracing::debug!("  {}", option);
                    if option.contains("ENABLE_FTS5") {
                        has_fts5 = true;
                    }
                }
            }
            if has_fts5 {
                tracing::debug!("✅ FTS5 is enabled in SQLite compilation");
            } else {
                tracing::warn!("❌ FTS5 is not enabled in SQLite compilation");
            }
        }
        Err(e) => {
            tracing::warn!("Could not check SQLite compile options: {:?}", e);
        }
    }
}

/// 测试Signal tokenizer是否可用
async fn test_signal_tokenizer_availability(conn: &mut sqlx::SqliteConnection) {
    // 首先尝试注册Signal tokenizer
    register_signal_tokenizer_via_ffi(conn).await;

    // 创建一个临时表来测试Signal tokenizer
    let test_result = sqlx::query(
        "CREATE VIRTUAL TABLE IF NOT EXISTS test_signal_tokenizer USING fts5(content, tokenize='signal_tokenizer')"
    ).execute(&mut *conn).await;

    match test_result {
        Ok(_) => {
            tracing::debug!("✅ Signal tokenizer is available and working");

            // 清理测试表
            let _ = sqlx::query("DROP TABLE IF EXISTS test_signal_tokenizer")
                .execute(&mut *conn)
                .await;
        }
        Err(e) => {
            tracing::warn!("❌ Signal tokenizer not available: {:?}", e);
            tracing::debug!("Will fall back to standard FTS5 tokenizers");
        }
    }
}

/// 通过FFI直接注册Signal tokenizer
async fn register_signal_tokenizer_via_ffi(conn: &mut sqlx::SqliteConnection) {
    // 尝试通过SQL查询来正确初始化Signal tokenizer
    // 这样会获得正确的API routines指针
    let init_result = sqlx::query("SELECT load_extension('signal_tokenizer')")
        .fetch_optional(&mut *conn)
        .await;

    match init_result {
        Ok(_) => {
            tracing::debug!("Signal tokenizer loaded via load_extension");
        }
        Err(_) => {
            // 如果load_extension失败，尝试直接注册
            tracing::debug!("Attempting direct Signal tokenizer registration via static linking");

            // 使用auto extension机制
            tracing::debug!(
                "Signal tokenizer auto extension should have been registered at startup"
            );
        }
    }
}

/// 注册Signal tokenizer为SQLite自动扩展
fn register_signal_auto_extension() {
    unsafe {
        let result = sqlite3_auto_extension(signal_fts5_tokenizer_init);
        if result == 0 {
            // SQLITE_OK
            tracing::debug!("✅ Signal tokenizer registered as auto extension successfully");
        } else {
            tracing::warn!(
                "❌ Failed to register Signal tokenizer as auto extension: error code {}",
                result
            );
        }
    }
}

/*
/// 注册 Signal FTS5 扩展的辅助函数
async fn register_signal_fts5_extension(pool: &SqlitePool) -> Result<()> {
    // 获取 SQLite 数据库句柄
    let mut conn = pool.acquire().await.map_err(TagboxError::Database)?;

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
