use crate::{
    config::AppConfig,
    errors::TagboxError,
    types::FileEntry,
    utils::{calculate_file_hash_with_type, HashType},
};
use chrono::{DateTime, Utc};
use sea_query::{Expr, Iden, Query, SqliteQueryBuilder};
use sqlx::{Row, SqlitePool};
use std::path::Path;
use tokio::fs;
use walkdir::WalkDir;

#[derive(Iden)]
enum Files {
    Table,
    Id,
    RelativePath,
    CurrentHash,
    Size,
    Title,
    Hash,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
#[allow(dead_code)]
enum FileHistory {
    Table,
    Id,
    FileId,
    Operation,
    OldHash,
    NewHash,
    OldPath,
    NewPath,
    OldSize,
    NewSize,
    ChangedAt,
    ChangedBy,
    Reason,
}

pub struct FileValidator {
    pool: SqlitePool,
    config: AppConfig,
}

impl FileValidator {
    pub fn new(pool: SqlitePool, config: AppConfig) -> Self {
        Self { pool, config }
    }

    pub async fn validate_files_in_path(
        &self,
        path: &Path,
        recursive: bool,
    ) -> Result<Vec<ValidationResult>, TagboxError> {
        let mut results = Vec::new();

        if recursive {
            for entry in WalkDir::new(path)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    let result = self.validate_single_file(entry.path()).await?;
                    results.push(result);
                }
            }
        } else {
            let mut entries = fs::read_dir(path).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() {
                    let result = self.validate_single_file(&path).await?;
                    results.push(result);
                }
            }
        }

        Ok(results)
    }

    pub async fn validate_single_file(&self, path: &Path) -> Result<ValidationResult, TagboxError> {
        // 尝试获取相对路径，如果文件在 storage_dir 外部，则使用绝对路径
        let (relative_path, absolute_path) = if path.is_absolute() {
            if let Ok(rel) = path.strip_prefix(&self.config.import.paths.storage_dir) {
                (rel.to_string_lossy().to_string(), path.to_path_buf())
            } else {
                // 文件在 storage_dir 外部，使用绝对路径
                (path.to_string_lossy().to_string(), path.to_path_buf())
            }
        } else {
            // 相对路径，转换为绝对路径
            let abs = self.config.import.paths.storage_dir.join(path);
            (path.to_string_lossy().to_string(), abs)
        };

        let query = Query::select()
            .columns([Files::Id, Files::CurrentHash, Files::Size])
            .from(Files::Table)
            .and_where(Expr::col(Files::RelativePath).eq(&relative_path))
            .to_string(SqliteQueryBuilder);

        let row = sqlx::query(&query).fetch_optional(&self.pool).await?;

        match row {
            Some(row) => {
                let file_id: String = row.get(0);
                let stored_hash: String = row.get(1);
                let stored_size: i64 = row.get(2);

                let metadata = fs::metadata(&absolute_path).await?;
                let current_size = metadata.len() as i64;

                if current_size != stored_size {
                    return Ok(ValidationResult {
                        file_id: Some(file_id),
                        path: absolute_path.clone(),
                        status: ValidationStatus::SizeMismatch {
                            expected: stored_size,
                            actual: current_size,
                        },
                    });
                }

                let hash_type = HashType::from_string(&self.config.hash.algorithm)?;
                let current_hash = calculate_file_hash_with_type(&absolute_path, hash_type).await?;

                if current_hash != stored_hash {
                    Ok(ValidationResult {
                        file_id: Some(file_id),
                        path: absolute_path,
                        status: ValidationStatus::HashMismatch {
                            expected: stored_hash,
                            actual: current_hash,
                        },
                    })
                } else {
                    Ok(ValidationResult {
                        file_id: Some(file_id),
                        path: absolute_path,
                        status: ValidationStatus::Valid,
                    })
                }
            }
            None => Ok(ValidationResult {
                file_id: None,
                path: absolute_path,
                status: ValidationStatus::NotInDatabase,
            }),
        }
    }

    pub async fn update_file_hash(
        &self,
        file_id: &str,
        reason: &str,
    ) -> Result<FileEntry, TagboxError> {
        let query = Query::select()
            .columns([Files::RelativePath, Files::CurrentHash, Files::Size])
            .from(Files::Table)
            .and_where(Expr::col(Files::Id).eq(file_id))
            .to_string(SqliteQueryBuilder);

        let row = sqlx::query(&query)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| TagboxError::NotFound(format!("File with id {} not found", file_id)))?;

        let relative_path: String = row.get(0);
        let old_hash: String = row.get(1);
        let old_size: i64 = row.get(2);

        let full_path = self.config.import.paths.storage_dir.join(&relative_path);
        let metadata = fs::metadata(&full_path).await?;
        let new_size = metadata.len() as i64;

        let hash_type = HashType::from_string(&self.config.hash.algorithm)?;
        let new_hash = calculate_file_hash_with_type(&full_path, hash_type).await?;

        let mut tx = self.pool.begin().await?;

        let update_query = Query::update()
            .table(Files::Table)
            .values([
                (Files::CurrentHash, new_hash.clone().into()),
                (Files::Size, new_size.into()),
            ])
            .and_where(Expr::col(Files::Id).eq(file_id))
            .to_string(SqliteQueryBuilder);

        sqlx::query(&update_query).execute(&mut *tx).await?;

        let history_id = uuid::Uuid::new_v4().to_string();
        let insert_history = Query::insert()
            .into_table(FileHistory::Table)
            .columns([
                FileHistory::Id,
                FileHistory::FileId,
                FileHistory::Operation,
                FileHistory::OldHash,
                FileHistory::NewHash,
                FileHistory::OldSize,
                FileHistory::NewSize,
                FileHistory::Reason,
            ])
            .values_panic([
                history_id.into(),
                file_id.into(),
                "hash_update".into(),
                old_hash.into(),
                new_hash.into(),
                old_size.into(),
                new_size.into(),
                reason.into(),
            ])
            .to_string(SqliteQueryBuilder);

        sqlx::query(&insert_history).execute(&mut *tx).await?;

        tx.commit().await?;

        // Re-fetch the updated file entry
        let select_query = Query::select()
            .columns([
                Files::Id,
                Files::Title,
                Files::Hash,
                Files::CurrentHash,
                Files::CreatedAt,
                Files::UpdatedAt,
            ])
            .from(Files::Table)
            .and_where(Expr::col(Files::Id).eq(file_id))
            .to_string(SqliteQueryBuilder);

        let row = sqlx::query(&select_query).fetch_one(&self.pool).await?;

        // Construct FileEntry manually since it doesn't implement FromRow
        let id: String = row.get(0);
        let title: String = row.get(1);
        let hash: String = row.get(2);
        let current_hash: Option<String> = row.get(3);
        let created_at_str: String = row.get(4);
        let updated_at_str: String = row.get(5);

        // Parse timestamps
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| TagboxError::Database(sqlx::Error::Decode(Box::new(e))))?
            .with_timezone(&Utc);
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|e| TagboxError::Database(sqlx::Error::Decode(Box::new(e))))?
            .with_timezone(&Utc);

        Ok(FileEntry {
            id,
            title,
            hash,
            current_hash,
            created_at,
            updated_at,
            // Fill in other required fields with defaults
            authors: vec![],
            year: None,
            publisher: None,
            source: None,
            path: self.config.import.paths.storage_dir.join(&relative_path),
            original_path: None,
            original_filename: relative_path
                .split('/')
                .next_back()
                .unwrap_or("")
                .to_string(),
            category1: "未分类".to_string(),
            category2: None,
            category3: None,
            tags: vec![],
            summary: None,
            last_accessed: None,
            is_deleted: false,
            file_metadata: None,
            type_metadata: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub file_id: Option<String>,
    pub path: std::path::PathBuf,
    pub status: ValidationStatus,
}

#[derive(Debug, Clone)]
pub enum ValidationStatus {
    Valid,
    NotInDatabase,
    HashMismatch { expected: String, actual: String },
    SizeMismatch { expected: i64, actual: i64 },
    FileNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    async fn setup_test_env() -> (TempDir, SqlitePool, AppConfig) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create the database file
        std::fs::File::create(&db_path).unwrap();

        let pool = SqlitePool::connect(&format!("sqlite:{}", db_path.display()))
            .await
            .unwrap();

        // Create test schema
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS files (
                id TEXT PRIMARY KEY,
                initial_hash TEXT,
                current_hash TEXT,
                relative_path TEXT,
                filename TEXT,
                title TEXT NOT NULL,
                size INTEGER NOT NULL DEFAULT 0,
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
                type_metadata TEXT
            );
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let mut config = AppConfig::default();
        config.import.paths.storage_dir = temp_dir.path().to_path_buf();
        config.hash.algorithm = "sha256".to_string();

        (temp_dir, pool, config)
    }

    #[tokio::test]
    async fn test_validate_file_not_in_database() {
        let (temp_dir, pool, config) = setup_test_env().await;
        let validator = FileValidator::new(pool, config);

        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, b"test content").await.unwrap();

        let result = validator.validate_single_file(&test_file).await.unwrap();

        assert!(result.file_id.is_none());
        assert!(matches!(result.status, ValidationStatus::NotInDatabase));
    }

    #[tokio::test]
    async fn test_validate_files_recursive() {
        let (temp_dir, pool, mut config) = setup_test_env().await;

        // Create a data directory separate from the temp dir
        let data_dir = temp_dir.path().join("data");
        fs::create_dir(&data_dir).await.unwrap();
        config.import.paths.storage_dir = data_dir.clone();

        let validator = FileValidator::new(pool, config);

        let sub_dir = data_dir.join("subdir");
        fs::create_dir(&sub_dir).await.unwrap();

        fs::write(data_dir.join("file1.txt"), b"content1")
            .await
            .unwrap();
        fs::write(sub_dir.join("file2.txt"), b"content2")
            .await
            .unwrap();

        let results = validator
            .validate_files_in_path(&data_dir, true)
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_validate_files_non_recursive() {
        let (temp_dir, pool, mut config) = setup_test_env().await;

        // Create a data directory separate from the temp dir
        let data_dir = temp_dir.path().join("data");
        fs::create_dir(&data_dir).await.unwrap();
        config.import.paths.storage_dir = data_dir.clone();

        let validator = FileValidator::new(pool, config);

        let sub_dir = data_dir.join("subdir");
        fs::create_dir(&sub_dir).await.unwrap();

        fs::write(data_dir.join("file1.txt"), b"content1")
            .await
            .unwrap();
        fs::write(sub_dir.join("file2.txt"), b"content2")
            .await
            .unwrap();

        let results = validator
            .validate_files_in_path(&data_dir, false)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
    }
}
