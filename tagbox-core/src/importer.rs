use crate::config::AppConfig;
use crate::errors::{Result, TagboxError};
use crate::metainfo::MetaInfoExtractor;
use crate::pathgen::PathGenerator;
use crate::types::{FileEntry, ImportMetadata};
use crate::utils::{
    calculate_file_hash, current_time, ensure_dir_exists, generate_uuid, require_field,
    safe_copy_file,
};
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// 文件导入器
pub struct Importer {
    config: AppConfig,
    db_pool: SqlitePool,
    metainfo_extractor: MetaInfoExtractor,
    path_generator: PathGenerator,
}

impl Importer {
    /// 创建一个新的导入器
    pub fn new(config: AppConfig, db_pool: SqlitePool) -> Self {
        let metainfo_extractor = MetaInfoExtractor::new(config.clone());
        let path_generator = PathGenerator::new(config.clone());

        Self {
            config,
            db_pool,
            metainfo_extractor,
            path_generator,
        }
    }

    /// 从文件路径导入文件
    pub async fn import(&self, file_path: &Path) -> Result<FileEntry> {
        info!("开始导入文件: {}", file_path.display());

        // 1. 检查文件是否存在
        if !file_path.exists() {
            return Err(TagboxError::FileNotFound {
                path: file_path.to_path_buf(),
            });
        }

        // 2. 计算文件哈希
        let hash = calculate_file_hash(file_path).await?;
        debug!("文件哈希: {}", hash);

        // 3. 检查文件是否已存在（基于哈希）
        if let Some(existing_entry) = self.find_by_hash(&hash).await? {
            warn!(
                "文件已存在: {} (ID: {})",
                existing_entry.path.display(),
                existing_entry.id
            );
            return Ok(existing_entry);
        }

        // 4. 提取元数据
        let metadata = self.metainfo_extractor.extract(file_path).await?;

        // 5. 生成新的文件名和目标路径
        let original_filename = file_path
            .file_name()
            .ok_or_else(|| TagboxError::Config(format!("无法获取文件名: {}", file_path.display())))?
            .to_string_lossy()
            .to_string();

        let new_filename = self
            .path_generator
            .generate_filename(&original_filename, &metadata)?;

        let dest_path = self
            .path_generator
            .generate_path(&new_filename, &metadata)?;

        // 确保目标目录存在
        if let Some(parent) = dest_path.parent() {
            ensure_dir_exists(parent)?;
        }

        // 6. 复制文件到目标位置
        safe_copy_file(file_path, &dest_path).await?;

        // 7. 创建文件记录
        let file_entry = self
            .create_file_entry(file_path, &dest_path, &original_filename, &hash, &metadata)
            .await?;

        info!(
            "文件导入完成: {} -> {} (ID: {})",
            file_path.display(),
            dest_path.display(),
            file_entry.id
        );

        Ok(file_entry)
    }

    /// 根据哈希查找文件
    async fn find_by_hash(&self, hash_to_find: &str) -> Result<Option<FileEntry>> {
        // Define the local FileEntryDb struct for this query's mapping
        #[derive(sqlx::FromRow)]
        struct FileEntryDbRow {
            id: String,
            title: String,
            initial_hash: Option<String>,
            current_hash: Option<String>,
            relative_path: Option<String>,
            filename: Option<String>,
            year: Option<i64>,
            publisher: Option<String>,
            category_id: Option<String>,
            source_url: Option<String>,
            summaries: Option<String>,
            created_at: String,
            updated_at: String,
            is_deleted: i64,
            _deleted_at: Option<String>,
            file_metadata: Option<String>,
            type_metadata: Option<String>,
        }

        let maybe_row = sqlx::query_as!(
            FileEntryDbRow,
            r#"
            SELECT 
                id as "id!", title as "title!", initial_hash, current_hash, 
                relative_path, filename, year, publisher, category_id, source_url, summaries,
                created_at as "created_at!", updated_at as "updated_at!", is_deleted, deleted_at as "_deleted_at",
                file_metadata, type_metadata
            FROM files
            WHERE initial_hash = ?1 OR current_hash = ?1
            LIMIT 1
            "#,
            hash_to_find
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(TagboxError::Database)?;

        if let Some(db_row) = maybe_row {
            let authors = self.get_file_authors(&db_row.id).await?;
            let tags = self.get_file_tags(&db_row.id).await?;

            Ok(Some(FileEntry {
                id: db_row.id,
                title: db_row.title,
                authors,
                year: db_row.year.map(|y| y as i32),
                publisher: db_row.publisher,
                source: db_row.source_url,
                path: PathBuf::from(require_field(db_row.relative_path, "files.relative_path")?),
                original_path: None,
                original_filename: require_field(db_row.filename, "files.filename")?,
                hash: require_field(db_row.initial_hash, "files.initial_hash")?,
                current_hash: db_row.current_hash,
                category1: require_field(db_row.category_id, "files.category_id")?,
                category2: None,
                category3: None,
                tags,
                summary: db_row.summaries,
                created_at: DateTime::parse_from_rfc3339(&db_row.created_at)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&db_row.updated_at)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                last_accessed: None,
                is_deleted: db_row.is_deleted != 0,
                file_metadata: db_row.file_metadata.as_deref()
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok()),
                type_metadata: db_row.type_metadata.as_deref()
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok()),
            }))
        } else {
            Ok(None)
        }
    }

    /// 创建文件记录
    async fn create_file_entry(
        &self,
        original_path: &Path,
        dest_path: &Path,
        original_filename_str: &str,
        hash_val: &str,
        metadata: &ImportMetadata,
    ) -> Result<FileEntry> {
        let id = generate_uuid();
        let now_datetime = current_time();
        let now_str_rfc3339 = now_datetime.to_rfc3339();

        let relative_path_for_db = dest_path.to_string_lossy().into_owned();
        let filename_for_db = dest_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();

        // 转换JSON元数据为字符串
        let file_metadata_str = metadata.file_metadata.as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default());
        let type_metadata_str = metadata.type_metadata.as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default());

        sqlx::query!(
            r#"
            INSERT INTO files (
                id, title, initial_hash, current_hash, relative_path, filename,
                year, publisher, category_id, source_url, summaries,
                created_at, updated_at, is_deleted, deleted_at,
                file_metadata, type_metadata
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            id,
            metadata.title,
            hash_val,
            hash_val,
            relative_path_for_db,
            filename_for_db,
            metadata.year,
            metadata.publisher,
            metadata.category1,
            metadata.source,
            metadata.summary,
            now_str_rfc3339,
            now_str_rfc3339,
            0,
            Option::<String>::None,
            file_metadata_str,
            type_metadata_str
        )
        .execute(&self.db_pool)
        .await
        .map_err(TagboxError::Database)?;

        let mut authors_for_entry = Vec::new();
        for author_name in &metadata.authors {
            let author_id = self.find_or_create_author(author_name).await?;
            self.link_author_to_file(&id, &author_id).await?;
            authors_for_entry.push(author_name.clone());
        }

        let mut tags_for_entry = Vec::new();
        for tag_name in &metadata.tags {
            let tag_id = self.find_or_create_tag(tag_name).await?;
            self.link_tag_to_file(&id, &tag_id).await?;
            tags_for_entry.push(tag_name.clone());
        }

        for (key, value) in &metadata.additional_info {
            self.add_metadata_to_file(&id, key, value).await?;
        }

        Ok(FileEntry {
            id,
            title: metadata.title.clone(),
            authors: authors_for_entry,
            year: metadata.year,
            publisher: metadata.publisher.clone(),
            source: metadata.source.clone(),
            path: dest_path.to_path_buf(),
            original_path: Some(original_path.to_path_buf()),
            original_filename: original_filename_str.to_string(),
            hash: hash_val.to_string(),
            current_hash: Some(hash_val.to_string()),
            category1: metadata.category1.clone(),
            category2: metadata.category2.clone(),
            category3: metadata.category3.clone(),
            tags: tags_for_entry,
            summary: metadata.summary.clone(),
            created_at: now_datetime,
            updated_at: now_datetime,
            last_accessed: None,
            is_deleted: false,
            file_metadata: metadata.file_metadata.clone(),
            type_metadata: metadata.type_metadata.clone(),
        })
    }

    /// 添加作者到文件
    async fn link_author_to_file(&self, file_id: &str, author_id: &str) -> Result<()> {
        sqlx::query!(
            "INSERT OR IGNORE INTO file_authors (file_id, author_id) VALUES (?, ?)",
            file_id,
            author_id
        )
        .execute(&self.db_pool)
        .await
        .map_err(TagboxError::Database)?;
        Ok(())
    }

    /// 查找或创建作者
    async fn find_or_create_author(&self, author_name: &str) -> Result<String> {
        let maybe_author = sqlx::query!(
            r#"SELECT id as "id!" FROM authors WHERE name = ?"#, // id! ensures String
            author_name
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(TagboxError::Database)?;

        if let Some(author) = maybe_author {
            return Ok(author.id);
        }

        let author_id = generate_uuid();
        let now_str = current_time().to_rfc3339();
        sqlx::query!(
            "INSERT INTO authors (id, name, created_at, updated_at) VALUES (?, ?, ?, ?)",
            author_id,
            author_name,
            now_str,
            now_str
        )
        .execute(&self.db_pool)
        .await
        .map_err(TagboxError::Database)?;
        Ok(author_id)
    }

    /// 添加标签到文件
    async fn link_tag_to_file(&self, file_id: &str, tag_id: &str) -> Result<()> {
        sqlx::query!(
            "INSERT OR IGNORE INTO file_tags (file_id, tag_id) VALUES (?, ?)",
            file_id,
            tag_id
        )
        .execute(&self.db_pool)
        .await
        .map_err(TagboxError::Database)?;
        Ok(())
    }

    /// 查找或创建标签
    async fn find_or_create_tag(&self, tag_name: &str) -> Result<String> {
        let maybe_tag = sqlx::query!(r#"SELECT id as "id!" FROM tags WHERE name = ?"#, tag_name)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(TagboxError::Database)?;

        if let Some(tag) = maybe_tag {
            return Ok(tag.id);
        }

        let id = generate_uuid();
        let now_str = current_time().to_rfc3339();
        sqlx::query!(
            r#"
            INSERT INTO tags (id, name, path, created_at, is_deleted, parent_id)
            VALUES (?, ?, ?, ?, ?, NULL) 
            "#,
            id,
            tag_name,
            tag_name,
            now_str,
            0
        )
        .execute(&self.db_pool)
        .await
        .map_err(TagboxError::Database)?;
        Ok(id)
    }

    /// 添加元数据到文件
    async fn add_metadata_to_file(&self, file_id: &str, key: &str, value: &str) -> Result<()> {
        sqlx::query!(
            "INSERT INTO file_metadata (file_id, key, value) VALUES (?, ?, ?)",
            file_id,
            key,
            value
        )
        .execute(&self.db_pool)
        .await
        .map_err(TagboxError::Database)?;
        Ok(())
    }

    /// 获取文件作者
    async fn get_file_authors(&self, file_id: &str) -> Result<Vec<String>> {
        let author_names = sqlx::query!(
            // In authors.rs, get_author fetches names for aliases.
            // Here, we just need the names of authors directly linked to the file.
            "SELECT name FROM authors a JOIN file_authors fa ON a.id = fa.author_id WHERE fa.file_id = ?",
            file_id
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(TagboxError::Database)?
        .into_iter()
        .map(|row| row.name) // row.name is String as it's NOT NULL in DB
        .collect();
        Ok(author_names)
    }

    /// 获取文件标签
    async fn get_file_tags(&self, file_id: &str) -> Result<Vec<String>> {
        let tag_names = sqlx::query!(
            "SELECT name FROM tags t JOIN file_tags ft ON t.id = ft.tag_id WHERE ft.file_id = ?",
            file_id
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(TagboxError::Database)?
        .into_iter()
        .map(|row| row.name) // row.name is String as it's NOT NULL in DB
        .collect();
        Ok(tag_names)
    }
}
