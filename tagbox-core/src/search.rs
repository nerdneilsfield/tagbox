use crate::config::AppConfig;
use crate::errors::{Result, TagboxError};
use crate::types::{FileEntry, QueryParam, SearchOptions, SearchResult};
use crate::utils::require_field;
use sqlx::{
    query::Query, sqlite::SqliteArguments, sqlite::SqlitePoolOptions, Arguments, Row, SqlitePool,
};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// 查询解析器和执行器
pub struct Searcher {
    config: AppConfig,
    db_pool: SqlitePool,
    fts5_signal_available: bool,
}

impl Searcher {
    /// 创建一个新的搜索器
    pub async fn new(config: AppConfig, db_pool: SqlitePool) -> Self {
        // 检查 Signal FTS5 是否可用
        let fts5_signal_available = Self::check_signal_fts5_available(&db_pool).await;

        if fts5_signal_available {
            info!("Signal FTS5 分词器可用，将启用高级模糊搜索");
        } else {
            info!("Signal FTS5 分词器不可用，将使用标准搜索");
        }

        Self {
            config,
            db_pool,
            fts5_signal_available,
        }
    }

    /// 检查 Signal FTS5 扩展是否可用
    async fn check_signal_fts5_available(pool: &SqlitePool) -> bool {
        // 测试查询，尝试使用 signal_cjk 分词器
        let result = sqlx::query("SELECT fts5(?);")
            .bind("signal_cjk")
            .execute(pool)
            .await;

        match result {
            Ok(_) => true,
            Err(_) => {
                // 二次检查是否有 signal_cjk 分词器
                let tokenizer_check = sqlx::query_scalar::<_, i64>(
                    "
                    SELECT COUNT(*) FROM sqlite_master 
                    WHERE type='table' AND name='files_fts' AND sql LIKE '%signal_cjk%'
                ",
                )
                .fetch_one(pool)
                .await;

                // 如果能找到使用该分词器的表，认为它可用
                tokenizer_check.unwrap_or(0) > 0
            }
        }
    }

    /// 简单文本搜索
    pub async fn search(&self, query: &str) -> Result<Vec<FileEntry>> {
        let options = SearchOptions {
            offset: 0,
            limit: self.config.search.default_limit,
            sort_by: Some("updated_at".to_string()),
            sort_direction: Some("DESC".to_string()),
            include_deleted: false,
        };

        let result = self.search_advanced(query, Some(options)).await?;
        Ok(result.entries)
    }

    /// 初始化FTS5索引
    pub async fn init_fts(&self) -> Result<()> {
        // 确保FTS5扩展已启用
        sqlx::query("SELECT fts5(?);")
            .bind("dummy")
            .execute(&self.db_pool)
            .await
            .map_err(|e| {
                warn!("FTS5扩展可能未启用: {}", e);
                TagboxError::Database(e)
            })?;

        info!("FTS5扩展已启用");

        // 创建或重建FTS5虚拟表 (根据 Signal FTS5 可用性选择分词器)
        let tokenizer = if self.fts5_signal_available {
            "signal_cjk porter unicode61 remove_diacritics 1"
        } else {
            "unicode61 remove_diacritics 1"
        };

        let fts_sql = format!(
            r#"
            DROP TABLE IF EXISTS files_fts;
            CREATE VIRTUAL TABLE files_fts USING fts5(
                title, 
                authors,
                summaries,
                tags,
                content='files',
                content_rowid='rowid',
                tokenize='{}'
            );
            "#,
            tokenizer
        );

        sqlx::query(&fts_sql)
            .execute(&self.db_pool)
            .await
            .map_err(|e| TagboxError::Database(e))?;

        // 重建FTS索引
        self.rebuild_fts_index().await?;

        info!("FTS5索引已初始化");
        Ok(())
    }

    /// 重建FTS索引
    pub async fn rebuild_fts_index(&self) -> Result<()> {
        info!("开始重建FTS索引...");

        // 清空当前FTS索引
        sqlx::query("DELETE FROM files_fts;")
            .execute(&self.db_pool)
            .await
            .map_err(|e| TagboxError::Database(e))?;

        // 获取所有文件
        let file_ids: Vec<String> =
            sqlx::query_scalar("SELECT id FROM files WHERE is_deleted = 0;")
                .fetch_all(&self.db_pool)
                .await
                .map_err(|e| TagboxError::Database(e))?;

        info!("找到 {} 个文件需要索引", file_ids.len());

        // 为每个文件添加索引
        for file_id in file_ids {
            self.update_fts_for_file(&file_id).await?;
        }

        info!("FTS索引重建完成");
        Ok(())
    }

    /// 更新单个文件的FTS索引
    pub async fn update_fts_for_file(&self, file_id: &str) -> Result<()> {
        // 获取文件信息
        let file = sqlx::query!(
            r#"
            SELECT rowid, title, summaries FROM files WHERE id = ?
            "#,
            file_id
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| TagboxError::Database(e))?
        .ok_or_else(|| TagboxError::InvalidFileId(file_id.to_string()))?;

        // 获取作者
        let authors = self.get_file_authors(file_id).await?;
        let authors_text = authors.join(" ");

        // 获取标签
        let tags = self.get_file_tags(file_id).await?;
        let tags_text = tags.join(" ");

        // 删除旧索引
        sqlx::query!(
            r#"
            DELETE FROM files_fts WHERE rowid = ?
            "#,
            file.rowid
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| TagboxError::Database(e))?;

        // 添加新索引
        sqlx::query!(
            r#"
            INSERT INTO files_fts (rowid, title, authors, summaries, tags)
            VALUES (?, ?, ?, ?, ?)
            "#,
            file.rowid,
            file.title,
            authors_text,
            file.summaries,
            tags_text
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| TagboxError::Database(e))?;

        debug!("已更新文件 {} 的FTS索引", file_id);
        Ok(())
    }

    /// 高级搜索，支持更多选项
    pub async fn search_advanced(
        &self,
        query: &str,
        options: Option<SearchOptions>,
    ) -> Result<SearchResult> {
        let options = options.unwrap_or_else(|| SearchOptions {
            offset: 0,
            limit: self.config.search.default_limit,
            sort_by: None,
            sort_direction: None,
            include_deleted: false,
        });

        let parsed = self.parse_query(query)?;

        // 构建基本SQL查询
        let mut sql = String::from(
            r#"
            SELECT 
                f.id, f.title, f.original_filename, f.hash, f.current_hash,
                f.path, f.original_path, f.category1, f.category2, f.category3,
                f.summaries,
                f.created_at, f.updated_at, f.last_accessed, f.is_deleted
            FROM files f
            "#,
        );

        let mut count_sql = String::from("SELECT COUNT(*) as count FROM files f");
        let mut params = Vec::new();

        // 应用作者过滤条件
        if !parsed.authors.is_empty() {
            sql.push_str(
                r#"
                JOIN file_authors fa ON f.id = fa.file_id
                JOIN authors a ON fa.author_id = a.id
                "#,
            );
            count_sql.push_str(
                r#"
                JOIN file_authors fa ON f.id = fa.file_id
                JOIN authors a ON fa.author_id = a.id
                "#,
            );
        }

        // 应用标签过滤条件
        if !parsed.include_tags.is_empty() || !parsed.exclude_tags.is_empty() {
            sql.push_str(
                r#"
                LEFT JOIN file_tags ft ON f.id = ft.file_id
                LEFT JOIN tags t ON ft.tag_id = t.id
                "#,
            );
            count_sql.push_str(
                r#"
                LEFT JOIN file_tags ft ON f.id = ft.file_id
                LEFT JOIN tags t ON ft.tag_id = t.id
                "#,
            );
        }

        // WHERE 子句
        let mut where_clauses = Vec::new();

        // 处理标题搜索
        if let Some(title) = &parsed.title {
            where_clauses.push("f.title LIKE ?".to_string());
            params.push(format!("%{}%", title));
        }

        // 处理包含的标签
        if !parsed.include_tags.is_empty() {
            let placeholders = vec!["?"; parsed.include_tags.len()].join(", ");
            where_clauses.push(format!("t.name IN ({})", placeholders));
            for tag in &parsed.include_tags {
                params.push(tag.clone());
            }
        }

        // 处理排除的标签
        if !parsed.exclude_tags.is_empty() {
            where_clauses.push(format!(
                "f.id NOT IN (
                SELECT file_id FROM file_tags ft2 
                JOIN tags t2 ON ft2.tag_id = t2.id 
                WHERE t2.name IN ({})
            )",
                vec!["?"; parsed.exclude_tags.len()].join(", ")
            ));

            for tag in &parsed.exclude_tags {
                params.push(tag.clone());
            }
        }

        // 处理作者过滤
        if !parsed.authors.is_empty() {
            let placeholders = vec!["?"; parsed.authors.len()].join(", ");
            where_clauses.push(format!("a.name IN ({})", placeholders));
            for author in &parsed.authors {
                params.push(author.clone());
            }
        }

        // 处理年份范围
        if let Some(year) = parsed.year {
            where_clauses.push(
                "EXISTS (
                SELECT 1 FROM file_metadata fm 
                WHERE fm.file_id = f.id 
                AND fm.key = 'year' 
                AND fm.value = ?
            )"
                .to_string(),
            );
            params.push(year.to_string());
        }

        // 处理分类过滤
        if let Some(category) = &parsed.category {
            where_clauses
                .push("(f.category1 = ? OR f.category2 = ? OR f.category3 = ?)".to_string());
            params.push(category.clone());
            params.push(category.clone());
            params.push(category.clone());
        }

        // 排除已删除文件
        if !options.include_deleted {
            where_clauses.push("f.is_deleted = 0".to_string());
        }

        // 应用全文搜索
        if !parsed.text.is_empty() && self.config.search.enable_fts {
            // 使用FTS5进行全文搜索，根据是否启用 Signal-FTS5 选择查询语法
            where_clauses.push(
                "f.rowid IN (
                SELECT rowid FROM files_fts 
                WHERE files_fts MATCH ?
            )"
                .to_string(),
            );

            let fts_query = if self.fts5_signal_available {
                self.build_signal_fts5_query(&parsed.text)
            } else {
                self.build_standard_fts5_query(&parsed.text)
            };

            params.push(fts_query);
        } else if !parsed.text.is_empty() {
            // 回退到简单的LIKE搜索
            where_clauses.push("(f.title LIKE ? OR f.summaries LIKE ?)".to_string());
            params.push(format!("%{}%", parsed.text));
            params.push(format!("%{}%", parsed.text));
        }

        // 添加 WHERE 子句
        if !where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            count_sql.push_str(" WHERE ");
            sql.push_str(&where_clauses.join(" AND "));
            count_sql.push_str(&where_clauses.join(" AND "));
        }

        // 添加 GROUP BY 以防重复
        sql.push_str(" GROUP BY f.id");

        // 添加排序
        if let Some(sort_by) = &options.sort_by {
            let direction = options.sort_direction.as_deref().unwrap_or("ASC");

            // 特殊处理基于全文搜索相关性排序
            if sort_by == "relevance" && !parsed.text.is_empty() && self.config.search.enable_fts {
                sql.push_str(
                    " ORDER BY (
                    SELECT rank FROM files_fts 
                    WHERE files_fts.rowid = f.rowid 
                    AND files_fts MATCH ?
                ) DESC",
                );

                let fts_query = if self.fts5_signal_available {
                    self.build_signal_fts5_query(&parsed.text)
                } else {
                    self.build_standard_fts5_query(&parsed.text)
                };

                params.push(fts_query);
            } else {
                sql.push_str(&format!(" ORDER BY f.{} {}", sort_by, direction));
            }
        }

        // 添加分页
        sql.push_str(&format!(
            " LIMIT {} OFFSET {}",
            options.limit, options.offset
        ));

        debug!("执行SQL查询: {}", sql);
        debug!(
            "参数: {:?}",
            params
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<String>>()
        );

        let mut count_args = SqliteArguments::default(); // Use default() to initialize
        for p_val in params.iter().take(count_sql.matches('?').count()) {
            count_args.add(p_val);
        }
        let total_count: i64 = sqlx::query_scalar_with(&count_sql, count_args)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|e| TagboxError::Database(e))?;

        let mut main_args = SqliteArguments::default(); // Use default() to initialize
        for p_val in &params {
            main_args.add(p_val);
        }
        let rows = sqlx::query_with(&sql, main_args)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| TagboxError::Database(e))?;

        // 处理结果
        let mut entries = Vec::with_capacity(rows.len());
        for row in rows {
            let file_id: &str = row.get("id");

            // 获取作者
            let authors = self.get_file_authors(file_id).await?;

            // 获取标签
            let tags = self.get_file_tags(file_id).await?;

            // 获取额外元数据
            let metadata = self.get_file_metadata(file_id).await?;

            // 构建 FileEntry
            let entry = FileEntry {
                id: row.get("id"),
                title: row.get("title"),
                authors,
                year: metadata.get("year").and_then(|v| v.parse::<i32>().ok()),
                publisher: metadata.get("publisher").cloned(),
                source: metadata.get("source").cloned(),
                path: PathBuf::from(row.get::<String, _>("path")),
                original_path: row
                    .get::<Option<String>, _>("original_path")
                    .map(PathBuf::from),
                original_filename: row.get("original_filename"),
                hash: row.get("hash"),
                current_hash: row.get("current_hash"),
                category1: row.get("category1"),
                category2: row.get("category2"),
                category3: row.get("category3"),
                tags,
                summary: row.get("summaries"),
                created_at: chrono::DateTime::parse_from_rfc3339(row.get::<&str, _>("created_at"))
                    .unwrap_or_else(|_| chrono::Utc::now().into())
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(row.get::<&str, _>("updated_at"))
                    .unwrap_or_else(|_| chrono::Utc::now().into())
                    .with_timezone(&chrono::Utc),
                last_accessed: row.get::<Option<&str>, _>("last_accessed").and_then(|t| {
                    chrono::DateTime::parse_from_rfc3339(t)
                        .ok()
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                }),
                is_deleted: row.get::<i64, _>("is_deleted") != 0,
            };

            entries.push(entry);
        }

        Ok(SearchResult {
            entries,
            total_count: total_count as usize,
            offset: options.offset,
            limit: options.limit,
        })
    }

    /// 构建 Signal FTS5 特有的查询表达式 (支持 CJK 和拼音搜索)
    fn build_signal_fts5_query(&self, text: &str) -> String {
        // 对于Signal FTS5，我们可以使用其特殊的查询语法

        // 将查询文本分割为词语
        let terms: Vec<_> = text
            .split_whitespace()
            .filter(|term| !term.is_empty())
            .collect();

        if terms.is_empty() {
            return String::new();
        }

        // Signal FTS5支持各种复杂的查询操作
        let mut query_parts = Vec::new();

        // 1. 精确短语匹配（最高权重）
        if terms.len() > 1 {
            query_parts.push(format!("\"{}\"^10", text));
        }

        // 2. 个别词语匹配（较高权重）
        for term in &terms {
            // 对每个词使用 Signal 的拼音特性和模糊匹配

            // 精确匹配（高权重）
            query_parts.push(format!("{}^5", term));

            // 前缀匹配（中等权重）
            query_parts.push(format!("{}*^3", term));

            // 拼音匹配（适用于中文搜索，中低权重）
            // Signal FTS5会自动匹配拼音

            // 模糊匹配（低权重）
            if term.len() > 2 {
                query_parts.push(format!("NEAR({}^0.5, 2)", term));
            }
        }

        // 组合所有查询部分
        query_parts.join(" OR ")
    }

    /// 构建标准 FTS5 查询表达式
    fn build_standard_fts5_query(&self, text: &str) -> String {
        // 将查询文本分割为词语
        let terms: Vec<_> = text
            .split_whitespace()
            .filter(|term| !term.is_empty())
            .collect();

        if terms.is_empty() {
            return String::new();
        }

        // 为每个词添加前缀搜索和近似匹配
        let mut query_parts = Vec::new();

        // 精确匹配整个短语(优先级最高)
        if terms.len() > 1 {
            query_parts.push(format!("\"{}\"^10", text));
        }

        // 各个词语的匹配
        for term in &terms {
            // 精确匹配（优先级较高）
            query_parts.push(format!("{}^5", term));

            // 前缀匹配（优先级中等）
            query_parts.push(format!("{}*^2", term));

            // 模糊匹配 (创建一些变体, 优先级低)
            if term.len() > 3 {
                let fuzzy_terms = self.generate_fuzzy_terms(term);
                for fuzzy_term in fuzzy_terms {
                    query_parts.push(fuzzy_term);
                }
            }
        }

        // 组合所有查询部分
        query_parts.join(" OR ")
    }

    /// 生成标准模糊匹配项
    fn generate_fuzzy_terms(&self, term: &str) -> Vec<String> {
        let mut terms = Vec::new();
        let chars: Vec<char> = term.chars().collect();

        // 如果词太短，无法进行有效的模糊匹配
        if chars.len() < 4 {
            return vec![term.to_string()];
        }

        // 1. 删除一个字符的版本
        for i in 0..chars.len() {
            let mut new_term = String::new();
            for (j, &c) in chars.iter().enumerate() {
                if i != j {
                    new_term.push(c);
                }
            }
            if new_term.len() >= 3 {
                // 只保留足够长的词
                terms.push(new_term);
            }
        }

        // 2. 替换一个字符的版本 (简化为通配符)
        for i in 0..chars.len() {
            let mut new_term = String::new();
            for (j, &c) in chars.iter().enumerate() {
                if i != j {
                    new_term.push(c);
                } else {
                    new_term.push('?'); // SQLite FTS5 通配符，匹配任何单个字符
                }
            }
            terms.push(new_term);
        }

        // 3. 交换相邻字符的版本
        for i in 0..chars.len() - 1 {
            let mut new_chars = chars.clone();
            new_chars.swap(i, i + 1);
            terms.push(new_chars.iter().collect());
        }

        // 返回所有生成的模糊匹配项，每个都赋予较低的权重
        terms.iter().map(|t| format!("{}^0.5", t)).collect()
    }

    /// 解析查询DSL
    fn parse_query(&self, query: &str) -> Result<ParsedQuery> {
        let mut parsed = ParsedQuery::default();
        let mut text_parts = Vec::new();

        // 简单的词法分析
        for part in query.split_whitespace() {
            if part.starts_with("tag:") {
                let tag = part[4..].trim();
                if !tag.is_empty() {
                    parsed.include_tags.push(tag.to_string());
                }
            } else if part.starts_with("-tag:") {
                let tag = part[5..].trim();
                if !tag.is_empty() {
                    parsed.exclude_tags.push(tag.to_string());
                }
            } else if part.starts_with("author:") {
                let author = part[7..].trim();
                if !author.is_empty() {
                    parsed.authors.push(author.to_string());
                }
            } else if part.starts_with("year:") {
                let year = part[5..].trim();
                if let Ok(year_num) = year.parse::<i32>() {
                    parsed.year = Some(year_num);
                }
            } else if part.starts_with("category:") {
                let category = part[9..].trim();
                if !category.is_empty() {
                    parsed.category = Some(category.to_string());
                }
            } else if part.starts_with("title:") {
                let title = part[6..].trim();
                if !title.is_empty() {
                    parsed.title = Some(title.to_string());
                }
            } else {
                text_parts.push(part);
            }
        }

        parsed.text = text_parts.join(" ");

        debug!("解析后的查询: {:?}", parsed);
        Ok(parsed)
    }

    /// 获取文件作者
    async fn get_file_authors(&self, file_id: &str) -> Result<Vec<String>> {
        let authors = sqlx::query!(
            r#"
            SELECT a.name
            FROM authors a
            JOIN file_authors fa ON a.id = fa.author_id
            WHERE fa.file_id = ?
            "#,
            file_id
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| TagboxError::Database(e))?;

        Ok(authors.iter().map(|a| a.name.clone()).collect())
    }

    /// 获取文件标签
    async fn get_file_tags(&self, file_id: &str) -> Result<Vec<String>> {
        let tags = sqlx::query!(
            r#"
            SELECT t.name
            FROM tags t
            JOIN file_tags ft ON t.id = ft.tag_id
            WHERE ft.file_id = ?
            "#,
            file_id
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| TagboxError::Database(e))?;

        Ok(tags.iter().map(|t| t.name.clone()).collect())
    }

    /// 获取文件元数据
    async fn get_file_metadata(&self, file_id: &str) -> Result<HashMap<String, String>> {
        let metadata = sqlx::query!(
            r#"
            SELECT key, value FROM file_metadata WHERE file_id = ?
            "#,
            file_id
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| TagboxError::Database(e))?;

        let mut map = HashMap::new();
        for row in metadata {
            map.insert(row.key, require_field(row.value, "file_metadata.value")?);
        }

        Ok(map)
    }

    /// 执行模糊文本搜索
    pub async fn fuzzy_search(
        &self,
        text: &str,
        options: Option<SearchOptions>,
    ) -> Result<SearchResult> {
        // 对于空查询，返回所有文件
        if text.is_empty() {
            return self.search_advanced("", options).await;
        }

        // 基于 Signal-FTS5 可用性选择不同的模糊搜索方法
        if self.fts5_signal_available {
            // 使用 Signal-FTS5 的特殊模糊搜索能力
            let query = format!("signal_fuzzy:{}", text);
            self.search_advanced(&query, options).await
        } else {
            // 回退到标准模糊搜索方法
            let sanitized_text = text
                .split_whitespace()
                .map(|term| format!("*{}*", term))
                .collect::<Vec<_>>()
                .join(" ");

            debug!("生成的模糊查询: {}", sanitized_text);
            self.search_advanced(&sanitized_text, options).await
        }
    }
}

/// 解析后的查询结构
#[derive(Debug, Default)]
struct ParsedQuery {
    text: String,
    title: Option<String>,
    include_tags: Vec<String>,
    exclude_tags: Vec<String>,
    authors: Vec<String>,
    year: Option<i32>,
    category: Option<String>,
}
