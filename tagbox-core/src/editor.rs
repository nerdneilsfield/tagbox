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

        if let Some(category_val) = &update.category1 {
            // Changed from category_id, using suggestion
            updates.push("category_id = ?".to_string()); // DB column is category_id
            params.push(QueryParam::String(category_val.clone()));
        }
        // Remove category2 and category3 as they are not in FileUpdateRequest or files table
        // if let Some(category2) = &update.category2 {
        //     updates.push("category2 = ?".to_string());
        //     params.push(QueryParam::String(category2.clone()));
        // }
        // if let Some(category3) = &update.category3 {
        //     updates.push("category3 = ?".to_string());
        //     params.push(QueryParam::String(category3.clone()));
        // }

        if let Some(summary_val) = &update.summary {
            // Changed from summaries, using suggestion
            updates.push("summary = ?".to_string()); // DB column is summary
            params.push(QueryParam::String(summary_val.clone()));
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
        //     // QueryParam::Float was removed as it's not in QueryParam enum
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
            // Assuming tags table has (id, name, created_at, category_id (optional))
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

        Ok(std::path::PathBuf::from(require_field(
            file_path.relative_path,
            "files.relative_path",
        )?))
    }

    /// 获取文件信息
    pub async fn get_file(&self, file_id: &str) -> Result<crate::types::FileEntry> {
        use chrono::{DateTime, Utc};

        // 查询文件基本信息
        let file_row = sqlx::query!(
            r#"
            SELECT 
                id, title, initial_hash, current_hash, relative_path, filename,
                year, publisher, category_id, source_url, summary,
                created_at, updated_at, is_deleted, deleted_at
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
            path: std::path::PathBuf::from(require_field(
                file_row.relative_path,
                "files.relative_path",
            )?),
            original_path: None,
            original_filename: require_field(file_row.filename, "files.filename")?,
            hash: require_field(file_row.initial_hash, "files.initial_hash")?,
            current_hash: file_row.current_hash,
            category1: require_field(file_row.category_id, "files.category_id")?,
            category2: None,
            category3: None,
            tags,
            summary: file_row.summary,
            created_at: DateTime::parse_from_rfc3339(&file_row.created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&file_row.updated_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            last_accessed: None,
            is_deleted: file_row.is_deleted != 0,
            file_metadata: None, // TODO: 从数据库加载
            type_metadata: None, // TODO: 从数据库加载
        })
    }
}
