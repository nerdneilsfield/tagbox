use crate::errors::{Result, TagboxError};
use crate::types::Author;
use crate::utils::{current_time, generate_uuid, require_field};
use sqlx::SqlitePool;
use std::collections::HashMap;
use tracing::info;

/// 作者管理器
pub struct AuthorManager {
    db_pool: SqlitePool,
}

impl AuthorManager {
    /// 创建一个新的作者管理器
    pub fn new(db_pool: SqlitePool) -> Self {
        Self { db_pool }
    }

    /// 获取作者信息
    pub async fn get_author(&self, author_id_param: &str) -> Result<Author> {
        let author_row = sqlx::query!(
            r#"
            SELECT id, name, created_at, updated_at
            FROM authors
            WHERE id = ?
            "#,
            author_id_param
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(TagboxError::Database)?
        .ok_or_else(|| TagboxError::Config(format!("作者ID不存在: {}", author_id_param)))?;

        // 获取别名. The author_aliases table uses (alias_id, canonical_id).
        // alias_id is the ID of an author entry that is an alias.
        // canonical_id is the ID of the main author entry it points to.
        // To get all names that are aliases FOR `author_id_param` (which is a canonical_id):
        // we need to select `authors.name` where `authors.id` is in `author_aliases.alias_id`
        // and `author_aliases.canonical_id` is `author_id_param`.
        // OR, if `authors.aliases` JSON field is the primary source for alias strings, use that.
        // The current query `SELECT alias FROM author_aliases WHERE author_id = ?` is problematic
        // because `author_aliases` doesn't have an `alias` text column nor a simple `author_id` column.
        // It has `alias_id` (FK to authors) and `canonical_id` (FK to authors).
        // Assuming we want to list names of other Author entries that are marked as aliases for this one:
        let alias_names = sqlx::query!(
            r#"
            SELECT a.name FROM authors a JOIN author_aliases aa ON a.id = aa.alias_id
            WHERE aa.canonical_id = ?
            "#,
            author_id_param
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(TagboxError::Database)?
        .into_iter()
        .map(|row| row.name) // row.name is String (NOT NULL in db)
        .collect();

        let metadata = self.get_author_metadata(author_id_param).await?;

        Ok(Author {
            id: require_field(author_row.id, "authors.id")?,
            name: author_row.name,
            aliases: alias_names, // Use the fetched alias names
            metadata: Some(metadata),
        })
    }

    /// 创建新作者
    pub async fn create_author(&self, name: &str, aliases: &[String]) -> Result<Author> {
        // 检查是否已存在同名作者
        let existing = sqlx::query!(
            r#"
            SELECT id FROM authors WHERE name = ?
            "#,
            name
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(TagboxError::Database)?;

        if let Some(author) = existing {
            return self
                .get_author(&require_field(author.id, "authors.id")?)
                .await;
        }

        // 创建新作者
        let id = generate_uuid();
        let now = current_time().to_rfc3339();

        sqlx::query!(
            r#"
            INSERT INTO authors (id, name, created_at, updated_at)
            VALUES (?, ?, ?, ?)
            "#,
            id,
            name,
            now,
            now
        )
        .execute(&self.db_pool)
        .await
        .map_err(TagboxError::Database)?;

        // 添加别名
        for alias in aliases {
            self.add_author_alias(&id, alias).await?;
        }

        self.get_author(&id).await
    }

    /// 添加作者别名
    pub async fn add_author_alias(
        &self,
        canonical_author_id: &str,
        alias_author_id: &str,
    ) -> Result<()> {
        // This function now should take two author IDs: the main one (canonical) and the one to be marked as alias.
        // The `alias` string parameter is removed.
        // We need to ensure both authors exist.
        // Then, insert into author_aliases (alias_id, canonical_id).

        // Check if alias_author_id is already an alias for someone, or if it's already a canonical_id for another alias.
        let existing_alias_mapping = sqlx::query!(
            "SELECT canonical_id FROM author_aliases WHERE alias_id = ?",
            alias_author_id
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(TagboxError::Database)?;

        if let Some(existing_map) = existing_alias_mapping {
            // existing_map.canonical_id is String (NOT NULL in db)
            if existing_map.canonical_id == canonical_author_id {
                // Direct comparison for String
                return Ok(()); // Already correctly mapped
            } else {
                return Err(TagboxError::Config(format!(
                    "Author {} is already an alias for {}.",
                    alias_author_id, existing_map.canonical_id
                )));
            }
        }

        let now = current_time().to_rfc3339();
        let note_val = "Manually added alias".to_string(); // Bind to a variable
        sqlx::query!(
            r#"
            INSERT INTO author_aliases (alias_id, canonical_id, merged_at, note)
            VALUES (?, ?, ?, ?)
            "#,
            alias_author_id,
            canonical_author_id,
            now,
            note_val // Use the bound variable directly (not wrapped in Some)
        )
        .execute(&self.db_pool)
        .await
        .map_err(TagboxError::Database)?;

        Ok(())
    }

    /// 合并两个作者
    pub async fn merge_authors(&self, source_id: &str, target_id: &str) -> Result<()> {
        // 检查两个作者是否都存在
        let source_exists = sqlx::query!(
            r#"SELECT COUNT(*) as count FROM authors WHERE id = ?"#,
            source_id
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(TagboxError::Database)?
        .count
            > 0;

        let target_exists = sqlx::query!(
            r#"SELECT COUNT(*) as count FROM authors WHERE id = ?"#,
            target_id
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(TagboxError::Database)?
        .count
            > 0;

        if !source_exists || !target_exists {
            return Err(TagboxError::Config("源作者或目标作者不存在".to_string()));
        }

        // 开始事务
        let mut tx = self.db_pool.begin().await.map_err(TagboxError::Database)?;

        // 1. 移动文件-作者关联
        sqlx::query!(
            r#"
            INSERT OR IGNORE INTO file_authors (file_id, author_id)
            SELECT file_id, ? FROM file_authors WHERE author_id = ?
            "#,
            target_id,
            source_id
        )
        .execute(&mut *tx)
        .await
        .map_err(TagboxError::Database)?;

        // 2. 移动作者别名: Update existing entries in `author_aliases` where `alias_id` or `canonical_id` is `source_id`.
        //    - If `alias_id` = `source_id`, update it to `target_id` (if `target_id` isn't already an alias itself or the canonical_id).
        //      This is complex because an alias cannot become an alias for another alias directly through merge like this.
        //      Usually, you re-point things that pointed TO source_id, or aliases OF source_id.
        //    - Point all aliases of `source_id` to `target_id`:
        //      UPDATE author_aliases SET canonical_id = ? WHERE canonical_id = ?
        //    - Mark `source_id` itself as an alias of `target_id`.
        sqlx::query!(
            r#"
            UPDATE author_aliases SET canonical_id = ? WHERE canonical_id = ?
            "#,
            target_id, // New canonical_id
            source_id  // Old canonical_id
        )
        .execute(&mut *tx)
        .await
        .map_err(TagboxError::Database)?;

        // 3. 记录合并操作: Mark source_id as an alias of target_id.
        //    The old query `INSERT INTO author_aliases (alias, author_id) SELECT name, ? FROM authors WHERE id = ?`
        //    was trying to use `authors.name` as an alias string, which is not how `author_aliases` works.
        let now = current_time().to_rfc3339();
        let note_string = format!("Merged from {}", source_id);
        sqlx::query!(
            r#"
            INSERT OR IGNORE INTO author_aliases (alias_id, canonical_id, merged_at, note)
            VALUES (?, ?, ?, ?)
            "#,
            source_id,
            target_id,
            now,
            note_string // Use the bound variable directly (not wrapped in Some)
        )
        .execute(&mut *tx)
        .await
        .map_err(TagboxError::Database)?;

        // 4. 删除源作者 (authors.is_deleted should be set to true instead of DELETE)
        // Or, if authors table `aliases` JSON field was the source of truth for alias strings, that needs update.
        // For now, assuming we are logically deleting by marking source_id as an alias and potentially deactivating it.
        // The original authors.delete is fine if that's the desired behavior after merging.
        sqlx::query!(r#"DELETE FROM authors WHERE id = ?"#, source_id)
            .execute(&mut *tx)
            .await
            .map_err(TagboxError::Database)?;

        // 提交事务
        tx.commit().await.map_err(TagboxError::Database)?;

        info!("作者 {} 已合并到 {}", source_id, target_id);
        Ok(())
    }

    /// 获取作者元数据
    async fn get_author_metadata(&self, _author_id: &str) -> Result<HashMap<String, String>> {
        // 本实现中暂不存储作者元数据，返回空Map
        // 未来可以扩展实现
        Ok(HashMap::new())
    }

    /// 查找可能的重复作者
    pub async fn find_duplicate_authors(&self) -> Result<Vec<(String, String, f32)>> {
        // 简单实现：查找名称相似的作者
        let authors_rows = sqlx::query!(
            r#"
            SELECT id as "id!", name FROM authors
            "#
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(TagboxError::Database)?;

        let mut potential_matches = Vec::new();

        for (i, author1) in authors_rows.iter().enumerate() {
            for author2 in authors_rows.iter().skip(i + 1) {
                let similarity = self.compute_name_similarity(&author1.name, &author2.name);

                if similarity > 0.8 {
                    potential_matches.push((
                        author1.id.clone(), // id is now String due to "id!"
                        author2.id.clone(), // id is now String
                        similarity,
                    ));
                }
            }
        }

        Ok(potential_matches)
    }

    /// 计算名称相似度的简单实现
    fn compute_name_similarity(&self, name1: &str, name2: &str) -> f32 {
        if name1 == name2 {
            return 1.0;
        }

        // 计算最长公共子序列长度
        let name1_len = name1.len();
        let name2_len = name2.len();

        if name1_len == 0 || name2_len == 0 {
            return 0.0;
        }

        // 去除空格并转为小写进行比较
        let name1_norm: String = name1
            .to_lowercase()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        let name2_norm: String = name2
            .to_lowercase()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();

        // 如果归一化后相同
        if name1_norm == name2_norm {
            return 0.95;
        }

        // 一个名称是否是另一个的前缀或后缀
        if name1_norm.starts_with(&name2_norm) || name2_norm.starts_with(&name1_norm) {
            return 0.85;
        }

        if name1_norm.ends_with(&name2_norm) || name2_norm.ends_with(&name1_norm) {
            return 0.85;
        }

        // 检查姓氏+名字首字母
        let name1_parts: Vec<&str> = name1.split_whitespace().collect();
        let name2_parts: Vec<&str> = name2.split_whitespace().collect();

        if name1_parts.len() > 1 && name2_parts.len() > 1 {
            // 检查姓相同
            if name1_parts.last() == name2_parts.last() {
                // 检查名字首字母
                let name1_first_initial = name1_parts[0].chars().next();
                let name2_first_initial = name2_parts[0].chars().next();

                if name1_first_initial == name2_first_initial {
                    return 0.9;
                }
            }
        }

        // 简单的字符匹配比例
        let common_chars = name1_norm
            .chars()
            .filter(|c| name2_norm.contains(*c))
            .count();

        common_chars as f32 * 2.0 / (name1_norm.len() + name2_norm.len()) as f32
    }
}
