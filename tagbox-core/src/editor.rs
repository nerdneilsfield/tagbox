// 在 update_file 方法中添加 FTS 索引更新逻辑

use crate::errors::{Result, TagboxError}; // Assuming Result and TagboxError are in errors.rs
use crate::types::{FileUpdateRequest, QueryParam}; // Assuming FileUpdateRequest is in types.rs
use crate::utils::{current_time, require_field};
use sqlx::{sqlite::SqliteArguments, Arguments, SqlitePool}; // Assuming current_time is in utils.rs

pub struct Editor {
    db_pool: SqlitePool,
    // Add other necessary fields for Editor, e.g., config, other managers
}

impl Editor {
    pub fn new(db_pool: SqlitePool) -> Self {
        Self { db_pool }
    }

    pub async fn update_file(&self, file_id: &str, update: FileUpdateRequest) -> Result<()> {
        // 检查文件是否存在
        let exists = sqlx::query!(
            r#"
            SELECT id FROM files WHERE id = ?
            "#,
            file_id
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(TagboxError::Database)?
        .is_some();

        if !exists {
            return Err(TagboxError::InvalidFileId(file_id.to_string()));
        }

        let now = current_time().to_rfc3339();
        let mut updates = Vec::new();
        let mut params = Vec::new(); // This will hold QueryParam values

        // 构建更新语句
        if let Some(title) = &update.title {
            updates.push("title = ?".to_string());
            params.push(QueryParam::String(title.clone()));
        }

        if let Some(category1) = &update.category1 {
            updates.push("category1 = ?".to_string());
            params.push(QueryParam::String(category1.clone()));
        }

        if let Some(category2) = &update.category2 {
            updates.push("category2 = ?".to_string());
            params.push(QueryParam::String(category2.clone()));
        }

        if let Some(category3) = &update.category3 {
            updates.push("category3 = ?".to_string());
            params.push(QueryParam::String(category3.clone()));
        }

        if let Some(summary_val) = &update.summary {
            updates.push("summary = ?".to_string());
            params.push(QueryParam::String(summary_val.clone()));
        }

        if let Some(full_text_val) = &update.full_text {
            updates.push("full_text = ?".to_string());
            params.push(QueryParam::String(full_text_val.clone()));
        }

        if let Some(is_deleted) = update.is_deleted {
            updates.push("is_deleted = ?".to_string());
            // is_deleted in DB is INTEGER, in FileUpdateRequest is bool
            params.push(QueryParam::Int(if is_deleted { 1 } else { 0 }));
        }

        // Fields below are removed based on "no field" errors for FileUpdateRequest
        // if let Some(source_url) = &update.source_url {
        //     updates.push("source_url = ?".to_string());
        //     params.push(QueryParam::String(source_url.clone()));
        // }

        // if let Some(thumbnail_url) = &update.thumbnail_url {
        //     updates.push("thumbnail_url = ?".to_string());
        //     params.push(QueryParam::String(thumbnail_url.clone()));
        // }

        // if let Some(file_size) = update.file_size {
        //     updates.push("file_size = ?".to_string());
        //     params.push(QueryParam::Int(file_size));
        // }

        // if let Some(year) = update.year { // year was Option<i32>
        //     updates.push("year = ?".to_string());
        //     params.push(QueryParam::Int(year.into())); // Convert i32 to i64 for QueryParam::Int
        // }

        // if let Some(rating) = update.rating {
        //     updates.push("rating = ?".to_string());
        //     // QueryParam::Float variant does not exist
        //     // params.push(QueryParam::Float(rating));
        // }

        // 添加更新时间
        updates.push("updated_at = ?".to_string());
        params.push(QueryParam::String(now.clone()));

        if !updates.is_empty() {
            let sql_stmt = format!("UPDATE files SET {} WHERE id = ?", updates.join(", "));

            let mut arguments = SqliteArguments::default(); // Use default()
            for p_val in &params {
                match p_val {
                    QueryParam::String(s) => arguments
                        .add(s)
                        .map_err(|e| TagboxError::Database(sqlx::Error::Encode(e)))?,
                    QueryParam::Int(i) => arguments
                        .add(i)
                        .map_err(|e| TagboxError::Database(sqlx::Error::Encode(e)))?,
                    // QueryParam::Float variant does not exist
                    // QueryParam::Float(f) => arguments.add(f),
                    // QueryParam::Bool variant does not exist
                    // QueryParam::Bool(b) => arguments.add(b),
                }
            }
            arguments
                .add(file_id)
                .map_err(|e| TagboxError::Database(sqlx::Error::Encode(e)))?; // Add file_id for the WHERE clause

            sqlx::query_with(&sql_stmt, arguments)
                .execute(&self.db_pool)
                .await
                .map_err(TagboxError::Database)?;
        }

        // 处理作者更新
        if let Some(authors) = &update.authors {
            // 清除旧作者关系
            sqlx::query!(
                r#"
                DELETE FROM file_authors WHERE file_id = ?
                "#,
                file_id
            )
            .execute(&self.db_pool)
            .await
            .map_err(TagboxError::Database)?;

            // 添加新作者
            for author_name in authors {
                // Assuming authors is Vec<String> of names. Need to get/create author IDs.
                // This part requires AuthorManager logic or direct insertion if simple name suffices for linking.
                // For now, let's assume self.add_author_to_file handles finding or creating author by name and linking.
                // If add_author_to_file is not available in Editor, we need a different approach.
                // Let's assume for now `add_author_to_file` is a helper method we might need to implement or call from AuthorManager.
                // For simplicity, if author_name is ID:
                // self.link_author_to_file(file_id, author_name).await?; // If author_name is ID
                // If author_name is a name string, we need to find/create and then link:
                // let author = self.author_manager.get_or_create_author(author_name).await?;
                // self.link_author_to_file(file_id, &author.id).await?;
                // This requires Editor to have access to AuthorManager or similar logic.

                // Placeholder: Assuming add_author_to_file is a method on Editor that handles this.
                self.add_author_to_file(file_id, author_name).await?;
            }

            // 因为作者改变，需要手动更新 FTS 索引的 authors 列
            let authors_text = authors.join(" ");
            sqlx::query!(
                r#"
                UPDATE files_fts SET authors = ?
                WHERE rowid = (SELECT rowid FROM files WHERE id = ?)
                "#,
                authors_text,
                file_id
            )
            .execute(&self.db_pool)
            .await
            .map_err(TagboxError::Database)?;
        }

        // 处理标签更新
        if let Some(tags) = &update.tags {
            // 清除旧标签关系
            sqlx::query!(
                r#"
                DELETE FROM file_tags WHERE file_id = ?
                "#,
                file_id
            )
            .execute(&self.db_pool)
            .await
            .map_err(TagboxError::Database)?;

            // 添加新标签
            for tag_name in tags {
                // Placeholder: Assuming add_tag_to_file is a method on Editor that handles this.
                self.add_tag_to_file(file_id, tag_name).await?;
            }

            // 因为标签改变，需要手动更新 FTS 索引的 tags 列
            let tags_text = tags.join(" ");
            sqlx::query!(
                r#"
                UPDATE files_fts SET tags = ?
                WHERE rowid = (SELECT rowid FROM files WHERE id = ?)
                "#,
                tags_text,
                file_id
            )
            .execute(&self.db_pool)
            .await
            .map_err(TagboxError::Database)?;
        }
        Ok(())
    }

    // Helper method placeholders - these would need actual implementation
    // or calls to respective managers (AuthorManager, TagManager)
    async fn add_author_to_file(&self, file_id: &str, author_name: &str) -> Result<()> {
        // 1. Check if author exists by name, get ID. If not, create author, get ID.
        let author = sqlx::query!("SELECT id FROM authors WHERE name = ?", author_name)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(TagboxError::Database)?;

        let author_id = if let Some(auth) = author {
            auth.id
        } else {
            let new_author_id = crate::utils::generate_uuid();
            let now = current_time().to_rfc3339();
            sqlx::query!(
                "INSERT INTO authors (id, name, created_at, updated_at) VALUES (?, ?, ?, ?)",
                new_author_id,
                author_name,
                now,
                now
            )
            .execute(&self.db_pool)
            .await
            .map_err(TagboxError::Database)?;
            Some(new_author_id) // Ensure compatible types
        };

        // 2. Link file_id and author_id in file_authors
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

    async fn add_tag_to_file(&self, file_id: &str, tag_name: &str) -> Result<()> {
        // 1. Check if tag exists by name, get ID. If not, create tag, get ID.
        let tag = sqlx::query!("SELECT id FROM tags WHERE name = ?", tag_name)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(TagboxError::Database)?;

        let tag_id = if let Some(t) = tag {
            t.id
        } else {
            let new_tag_id = crate::utils::generate_uuid();
            let now = current_time().to_rfc3339();
            // Tags table has (id, name, created_at)
            // Removed updated_at based on "table tags has no column named updated_at" error
            sqlx::query!(
                "INSERT INTO tags (id, name, path, created_at, is_deleted, parent_id) VALUES (?, ?, ?, ?, ?, NULL)",
                new_tag_id, tag_name, tag_name, now, 0
            )
            .execute(&self.db_pool)
            .await.map_err(TagboxError::Database)?;
            Some(new_tag_id) // Ensure compatible types
        };

        // 2. Link file_id and tag_id in file_tags
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

    /// 获取文件路径
    pub async fn get_file_path(&self, file_id: &str) -> Result<std::path::PathBuf> {
        let file_path = sqlx::query!(
            "SELECT relative_path FROM files WHERE id = ? AND is_deleted = 0",
            file_id
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(TagboxError::Database)?
        .ok_or_else(|| TagboxError::InvalidFileId(file_id.to_string()))?;

        Ok(std::path::PathBuf::from(file_path.relative_path))
    }

    /// 获取文件信息
    pub async fn get_file(&self, file_id: &str) -> Result<crate::types::FileEntry> {
        use chrono::{DateTime, Utc};

        // 查询文件基本信息
        let file_row = sqlx::query!(
            r#"
            SELECT
                id, title, initial_hash, current_hash, relative_path, filename,
                year, publisher, category1, category2, category3, source_url, summary, full_text,
                created_at, updated_at, is_deleted, deleted_at,
                file_metadata, type_metadata
            FROM files
            WHERE id = ?
            "#,
            file_id
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(TagboxError::Database)?
        .ok_or_else(|| TagboxError::InvalidFileId(file_id.to_string()))?;

        // 获取作者
        let authors = sqlx::query!(
            "SELECT name FROM authors a JOIN file_authors fa ON a.id = fa.author_id WHERE fa.file_id = ?",
            file_id
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(TagboxError::Database)?
        .into_iter()
        .map(|row| row.name)
        .collect();

        // 获取标签
        let tags = sqlx::query!(
            "SELECT name FROM tags t JOIN file_tags ft ON t.id = ft.tag_id WHERE ft.file_id = ?",
            file_id
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(TagboxError::Database)?
        .into_iter()
        .map(|row| row.name)
        .collect();

        Ok(crate::types::FileEntry {
            id: require_field(file_row.id, "files.id")?,
            title: file_row.title,
            authors,
            year: file_row.year.map(|y| y as i32),
            publisher: file_row.publisher,
            source: file_row.source_url,
            path: std::path::PathBuf::from(file_row.relative_path),
            original_path: None,
            original_filename: file_row.filename,
            hash: file_row.initial_hash,
            current_hash: file_row.current_hash,
            category1: file_row.category1.unwrap_or_default(),
            category2: file_row.category2,
            category3: file_row.category3,
            tags,
            summary: file_row.summary,
            full_text: file_row.full_text,
            created_at: DateTime::parse_from_rfc3339(&file_row.created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&file_row.updated_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            last_accessed: None,
            is_deleted: file_row.is_deleted != 0,
            file_metadata: file_row
                .file_metadata
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok()),
            type_metadata: file_row
                .type_metadata
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok()),
        })
    }

    /// 更新文件并可选择移动到新路径
    pub async fn update_file_with_move(
        &self,
        file_id: &str,
        update: FileUpdateRequest,
        auto_move: bool,
        config: &crate::config::AppConfig,
    ) -> Result<Option<std::path::PathBuf>> {
        // 获取更新前的文件信息
        let _old_file = self.get_file(file_id).await?;

        // 执行基本更新
        self.update_file(file_id, update.clone()).await?;

        // 如果需要移动且分类发生了变化
        if auto_move
            && (update.category1.is_some()
                || update.category2.is_some()
                || update.category3.is_some())
        {
            return self.move_file(file_id, config).await.map(Some);
        }

        Ok(None)
    }

    /// 根据当前分类移动文件到正确的路径
    pub async fn move_file(
        &self,
        file_id: &str,
        config: &crate::config::AppConfig,
    ) -> Result<std::path::PathBuf> {
        use crate::pathgen::PathGenerator;
        use crate::types::ImportMetadata;
        use std::fs;

        // 获取文件当前信息
        let file = self.get_file(file_id).await?;

        // 构建新的元数据用于路径生成
        let metadata = ImportMetadata {
            title: file.title.clone(),
            authors: file.authors.clone(),
            year: file.year,
            publisher: file.publisher.clone(),
            source: file.source.clone(),
            category1: file.category1.clone(),
            category2: file.category2.clone(),
            category3: file.category3.clone(),
            tags: file.tags.clone(),
            summary: file.summary.clone(),
            full_text: file.full_text.clone(),
            additional_info: std::collections::HashMap::new(),
            file_metadata: file.file_metadata.clone(),
            type_metadata: file.type_metadata.clone(),
        };

        // 生成新路径
        let path_generator = PathGenerator::new(config.clone());
        let new_filename = path_generator.generate_filename(&file.original_filename, &metadata)?;
        let new_path = path_generator.generate_path(&new_filename, &metadata)?;

        // 当前文件的绝对路径
        let old_absolute_path = config.import.paths.storage_dir.join(&file.path);

        // 如果路径没有改变，直接返回
        if new_path == old_absolute_path {
            return Ok(new_path);
        }

        // 创建目标目录
        if let Some(parent) = new_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                TagboxError::FileSystem(format!("Failed to create directory: {}", e))
            })?;
        }

        // 移动文件
        fs::rename(&old_absolute_path, &new_path)
            .map_err(|e| TagboxError::FileSystem(format!("Failed to move file: {}", e)))?;

        // 更新数据库中的路径
        let new_relative_path = new_path
            .strip_prefix(&config.import.paths.storage_dir)
            .map_err(|_| TagboxError::FileSystem("Invalid storage path".to_string()))?
            .to_string_lossy()
            .to_string();

        let updated_at = crate::utils::current_time().to_rfc3339();
        sqlx::query!(
            "UPDATE files SET relative_path = ?, updated_at = ? WHERE id = ?",
            new_relative_path,
            updated_at,
            file_id
        )
        .execute(&self.db_pool)
        .await
        .map_err(TagboxError::Database)?;

        Ok(new_path)
    }

    /// 检查单个文件路径是否需要重建
    pub async fn check_file_path(
        &self,
        file_id: &str,
        config: &crate::config::AppConfig,
    ) -> Result<Option<std::path::PathBuf>> {
        use crate::pathgen::PathGenerator;
        use crate::types::ImportMetadata;

        let file = self.get_file(file_id).await?;

        // 构建元数据
        let metadata = ImportMetadata {
            title: file.title.clone(),
            authors: file.authors.clone(),
            year: file.year,
            publisher: file.publisher.clone(),
            source: file.source.clone(),
            category1: file.category1.clone(),
            category2: file.category2.clone(),
            category3: file.category3.clone(),
            tags: file.tags.clone(),
            summary: file.summary.clone(),
            full_text: file.full_text.clone(),
            additional_info: std::collections::HashMap::new(),
            file_metadata: file.file_metadata.clone(),
            type_metadata: file.type_metadata.clone(),
        };

        // 生成应该的路径
        let path_generator = PathGenerator::new(config.clone());
        let expected_filename =
            path_generator.generate_filename(&file.original_filename, &metadata)?;
        let expected_path = path_generator.generate_path(&expected_filename, &metadata)?;

        // 当前文件的绝对路径
        let current_absolute_path = config.import.paths.storage_dir.join(&file.path);

        // 如果路径不同，返回预期路径
        if expected_path != current_absolute_path {
            Ok(Some(expected_path))
        } else {
            Ok(None)
        }
    }

    /// 重建单个文件的路径
    pub async fn rebuild_file_path(
        &self,
        file_id: &str,
        config: &crate::config::AppConfig,
    ) -> Result<Option<std::path::PathBuf>> {
        if let Some(_new_path) = self.check_file_path(file_id, config).await? {
            let moved_path = self.move_file(file_id, config).await?;
            Ok(Some(moved_path))
        } else {
            Ok(None)
        }
    }

    /// 重建所有文件的路径（支持并行）
    pub async fn rebuild_all_files(
        &self,
        config: &crate::config::AppConfig,
        dry_run: bool,
        progress_callback: Option<Box<dyn Fn(usize, usize) + Send + Sync>>,
    ) -> Result<Vec<(String, std::path::PathBuf, std::path::PathBuf)>> {
        use tokio::task::JoinSet;

        // 获取所有文件ID
        let file_ids: Vec<String> = sqlx::query!("SELECT id FROM files WHERE is_deleted = 0")
            .fetch_all(&self.db_pool)
            .await
            .map_err(TagboxError::Database)?
            .into_iter()
            .filter_map(|row| row.id)
            .collect();

        let total_files = file_ids.len();
        let mut results = Vec::new();
        let mut completed = 0;

        // 分批处理以避免过多并发
        let chunk_size = 10;
        for chunk in file_ids.chunks(chunk_size) {
            let mut join_set: JoinSet<
                Result<Option<(String, std::path::PathBuf, std::path::PathBuf)>>,
            > = JoinSet::new();

            for file_id in chunk {
                let file_id = file_id.clone();
                let config = config.clone();
                let db_pool = self.db_pool.clone();

                join_set.spawn(async move {
                    let editor = Editor::new(db_pool);
                    let current_path = editor.get_file(&file_id).await?.path;

                    match editor.check_file_path(&file_id, &config).await? {
                        Some(new_path) => {
                            let old_absolute = config.import.paths.storage_dir.join(&current_path);
                            if !dry_run {
                                editor.move_file(&file_id, &config).await?;
                            }
                            Ok(Some((file_id, old_absolute, new_path)))
                        }
                        None => Ok(None),
                    }
                });
            }

            // 等待当前批次完成
            while let Some(result) = join_set.join_next().await {
                match result {
                    Ok(Ok(Some(move_info))) => {
                        results.push(move_info);
                    }
                    Ok(Ok(None)) => {
                        // 文件不需要移动
                    }
                    Ok(Err(e)) => {
                        tracing::warn!("Failed to process file: {}", e);
                    }
                    Err(e) => {
                        tracing::warn!("Task failed: {}", e);
                    }
                }

                completed += 1;
                if let Some(ref callback) = progress_callback {
                    callback(completed, total_files);
                }
            }
        }

        Ok(results)
    }

    /// 获取文件信息用于编辑（包含更多详细信息）
    pub async fn get_file_for_edit(&self, file_id: &str) -> Result<crate::types::FileEntry> {
        self.get_file(file_id).await
    }

    /// 预览更改（返回更改摘要）
    pub fn preview_changes(
        &self,
        original: &crate::types::FileEntry,
        update: &FileUpdateRequest,
    ) -> Vec<String> {
        let mut changes = Vec::new();

        if let Some(title) = &update.title {
            if title != &original.title {
                changes.push(format!("Title: '{}' → '{}'", original.title, title));
            }
        }

        if let Some(authors) = &update.authors {
            if authors != &original.authors {
                changes.push(format!("Authors: {:?} → {:?}", original.authors, authors));
            }
        }

        if let Some(category1) = &update.category1 {
            if category1 != &original.category1 {
                changes.push(format!(
                    "Category1: '{}' → '{}'",
                    original.category1, category1
                ));
            }
        }

        if let Some(category2) = &update.category2 {
            if Some(category2) != original.category2.as_ref() {
                changes.push(format!(
                    "Category2: {:?} → {:?}",
                    original.category2, category2
                ));
            }
        }

        if let Some(category3) = &update.category3 {
            if Some(category3) != original.category3.as_ref() {
                changes.push(format!(
                    "Category3: {:?} → {:?}",
                    original.category3, category3
                ));
            }
        }

        if let Some(tags) = &update.tags {
            if tags != &original.tags {
                changes.push(format!("Tags: {:?} → {:?}", original.tags, tags));
            }
        }

        if let Some(summary) = &update.summary {
            if Some(summary) != original.summary.as_ref() {
                changes.push(format!("Summary: {:?} → {:?}", original.summary, summary));
            }
        }

        if let Some(year) = &update.year {
            if Some(*year) != original.year {
                changes.push(format!("Year: {:?} → {}", original.year, year));
            }
        }

        if let Some(publisher) = &update.publisher {
            if Some(publisher) != original.publisher.as_ref() {
                changes.push(format!(
                    "Publisher: {:?} → {:?}",
                    original.publisher, publisher
                ));
            }
        }

        if let Some(source) = &update.source {
            if Some(source) != original.source.as_ref() {
                changes.push(format!("Source: {:?} → {:?}", original.source, source));
            }
        }

        changes
    }
}
