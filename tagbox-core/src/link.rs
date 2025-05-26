use crate::errors::{Result, TagboxError};
use crate::types::RelationType;
use crate::utils::current_time;
use sqlx::SqlitePool;
use tracing::debug;

/// 文件关联管理器
pub struct LinkManager {
    db_pool: SqlitePool,
}

impl LinkManager {
    /// 创建一个新的关联管理器
    pub fn new(db_pool: SqlitePool) -> Self {
        Self { db_pool }
    }
    
    /// 创建文件之间的关联
    pub async fn create_link(&self, source_file_id: &str, target_file_id: &str, relation_str: Option<String>) -> Result<()> {
        // 检查两个文件是否都存在
        self.verify_files_exist(source_file_id, target_file_id).await?;
        
        if source_file_id == target_file_id {
            return Err(TagboxError::Config(
                "不能将文件与自身关联".to_string()
            ));
        }
        
        let relation_col_value = RelationType::from(relation_str).to_string();
        let now = current_time().to_rfc3339();
        
        let exists = sqlx::query!(
            r#"
            SELECT 1 as exists_flag FROM file_links
            WHERE (source_id = ? AND target_id = ?)
            OR (source_id = ? AND target_id = ?)
            "#,
            source_file_id, target_file_id,
            target_file_id, source_file_id
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| TagboxError::Database(e))?
        .is_some();
        
        if exists {
            debug!("关联已存在，更新关系类型");
            sqlx::query!(
                r#"
                UPDATE file_links SET relation = ?
                WHERE (source_id = ? AND target_id = ?)
                OR (source_id = ? AND target_id = ?)
                "#,
                relation_col_value,
                source_file_id, target_file_id,
                target_file_id, source_file_id
            )
            .execute(&self.db_pool)
            .await
            .map_err(|e| TagboxError::Database(e))?;
        } else {
            // 创建新关联
            sqlx::query!(
                r#"
                INSERT INTO file_links (source_id, target_id, relation, created_at)
                VALUES (?, ?, ?, ?)
                "#,
                source_file_id,
                target_file_id,
                relation_col_value,
                now
            )
            .execute(&self.db_pool)
            .await
            .map_err(|e| TagboxError::Database(e))?;
        }
        
        Ok(())
    }
    
    /// 删除文件之间的关联
    pub async fn remove_link(&self, source_file_id: &str, target_file_id: &str) -> Result<()> {
        let result = sqlx::query!(
            r#"
            DELETE FROM file_links
            WHERE (source_id = ? AND target_id = ?)
            OR (source_id = ? AND target_id = ?)
            "#,
            source_file_id, target_file_id,
            target_file_id, source_file_id
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| TagboxError::Database(e))?;
        
        if result.rows_affected() == 0 {
            return Err(TagboxError::LinkNotFound { 
                file_id_a: source_file_id.to_string(), 
                file_id_b: target_file_id.to_string() 
            });
        }
        
        Ok(())
    }
    
    /// 获取文件的所有关联
    pub async fn get_links_for_file(&self, file_id: &str) -> Result<Vec<(String, String, String)>> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                CASE 
                    WHEN source_id = ? THEN target_id
                    ELSE source_id
                END as linked_file_id,
                relation, 
                created_at 
            FROM file_links
            WHERE source_id = ? OR target_id = ?
            "#,
            file_id,
            file_id,
            file_id
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| TagboxError::Database(e))?;
        
        Ok(rows.into_iter()
            .map(|row| (
                row.linked_file_id,
                row.relation.unwrap_or_default(),
                row.created_at
            ))
            .collect())
    }
    
    /// 验证两个文件ID是否有效
    async fn verify_files_exist(&self, source_file_id: &str, target_file_id: &str) -> Result<()> {
        // 检查第一个文件
        let file_a_exists = sqlx::query!(
            r#"
            SELECT 1 as exists_flag FROM files WHERE id = ?
            "#,
            source_file_id
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| TagboxError::Database(e))?
        .is_some();
        
        if !file_a_exists {
            return Err(TagboxError::InvalidFileId(source_file_id.to_string()));
        }
        
        // 检查第二个文件
        let file_b_exists = sqlx::query!(
            r#"
            SELECT 1 as exists_flag FROM files WHERE id = ?
            "#,
            target_file_id
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| TagboxError::Database(e))?
        .is_some();
        
        if !file_b_exists {
            return Err(TagboxError::InvalidFileId(target_file_id.to_string()));
        }
        
        Ok(())
    }
}