use crate::errors::TagboxError;
use chrono::Utc;
use sea_query::{Expr, Iden, Query, SqliteQueryBuilder};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

#[derive(Iden)]
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

#[derive(Iden)]
enum FileAccessStats {
    Table,
    FileId,
    AccessCount,
    LastAccessedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHistoryEntry {
    pub id: String,
    pub file_id: String,
    pub operation: String,
    pub old_hash: Option<String>,
    pub new_hash: Option<String>,
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub old_size: Option<i64>,
    pub new_size: Option<i64>,
    pub changed_at: String,
    pub changed_by: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAccessStatsEntry {
    pub file_id: String,
    pub access_count: i64,
    pub last_accessed_at: String,
    pub created_at: String,
    pub updated_at: String,
}

pub struct FileHistoryManager {
    pool: SqlitePool,
}

impl FileHistoryManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn record_file_history(
        &self,
        file_id: &str,
        operation: FileOperation,
        changed_by: Option<&str>,
        reason: Option<&str>,
    ) -> Result<String, TagboxError> {
        let history_id = uuid::Uuid::new_v4().to_string();

        let mut columns = vec![FileHistory::Id, FileHistory::FileId, FileHistory::Operation];

        let mut values: Vec<sea_query::SimpleExpr> = vec![
            history_id.clone().into(),
            file_id.into(),
            operation.as_str().into(),
        ];

        match &operation {
            FileOperation::Create { hash, path, size } => {
                columns.extend([
                    FileHistory::NewHash,
                    FileHistory::NewPath,
                    FileHistory::NewSize,
                ]);
                values.extend([hash.clone().into(), path.clone().into(), (*size).into()]);
            }
            FileOperation::Update {
                old_hash,
                new_hash,
                old_size,
                new_size,
            } => {
                columns.extend([
                    FileHistory::OldHash,
                    FileHistory::NewHash,
                    FileHistory::OldSize,
                    FileHistory::NewSize,
                ]);
                values.extend([
                    old_hash.clone().into(),
                    new_hash.clone().into(),
                    (*old_size).into(),
                    (*new_size).into(),
                ]);
            }
            FileOperation::Move { old_path, new_path } => {
                columns.extend([FileHistory::OldPath, FileHistory::NewPath]);
                values.extend([old_path.clone().into(), new_path.clone().into()]);
            }
            FileOperation::Delete { hash, path, size } => {
                columns.extend([
                    FileHistory::OldHash,
                    FileHistory::OldPath,
                    FileHistory::OldSize,
                ]);
                values.extend([hash.clone().into(), path.clone().into(), (*size).into()]);
            }
            FileOperation::Access => {}
        }

        if let Some(by) = changed_by {
            columns.push(FileHistory::ChangedBy);
            values.push(by.into());
        }

        if let Some(r) = reason {
            columns.push(FileHistory::Reason);
            values.push(r.into());
        }

        let query = Query::insert()
            .into_table(FileHistory::Table)
            .columns(columns)
            .values_panic(values)
            .to_string(SqliteQueryBuilder);

        sqlx::query(&query).execute(&self.pool).await?;

        if matches!(operation, FileOperation::Access) {
            self.update_access_stats(file_id).await?;
        }

        Ok(history_id)
    }

    pub async fn get_file_history(
        &self,
        file_id: &str,
        limit: Option<u64>,
    ) -> Result<Vec<FileHistoryEntry>, TagboxError> {
        let mut query = Query::select()
            .columns([
                FileHistory::Id,
                FileHistory::FileId,
                FileHistory::Operation,
                FileHistory::OldHash,
                FileHistory::NewHash,
                FileHistory::OldPath,
                FileHistory::NewPath,
                FileHistory::OldSize,
                FileHistory::NewSize,
                FileHistory::ChangedAt,
                FileHistory::ChangedBy,
                FileHistory::Reason,
            ])
            .from(FileHistory::Table)
            .and_where(Expr::col(FileHistory::FileId).eq(file_id))
            .order_by(FileHistory::ChangedAt, sea_query::Order::Desc)
            .to_owned();

        if let Some(limit) = limit {
            query.limit(limit);
        }

        let query_str = query.to_string(SqliteQueryBuilder);
        let rows = sqlx::query(&query_str).fetch_all(&self.pool).await?;

        Ok(rows
            .into_iter()
            .map(|row| FileHistoryEntry {
                id: row.get(0),
                file_id: row.get(1),
                operation: row.get(2),
                old_hash: row.get(3),
                new_hash: row.get(4),
                old_path: row.get(5),
                new_path: row.get(6),
                old_size: row.get(7),
                new_size: row.get(8),
                changed_at: row.get(9),
                changed_by: row.get(10),
                reason: row.get(11),
            })
            .collect())
    }

    pub async fn get_access_stats(
        &self,
        file_id: &str,
    ) -> Result<Option<FileAccessStatsEntry>, TagboxError> {
        let query = Query::select()
            .columns([
                FileAccessStats::FileId,
                FileAccessStats::AccessCount,
                FileAccessStats::LastAccessedAt,
                FileAccessStats::CreatedAt,
                FileAccessStats::UpdatedAt,
            ])
            .from(FileAccessStats::Table)
            .and_where(Expr::col(FileAccessStats::FileId).eq(file_id))
            .to_string(SqliteQueryBuilder);

        let row = sqlx::query(&query).fetch_optional(&self.pool).await?;

        Ok(row.map(|row| FileAccessStatsEntry {
            file_id: row.get(0),
            access_count: row.get(1),
            last_accessed_at: row.get(2),
            created_at: row.get(3),
            updated_at: row.get(4),
        }))
    }

    async fn update_access_stats(&self, file_id: &str) -> Result<(), TagboxError> {
        let existing = self.get_access_stats(file_id).await?;
        let now = Utc::now().to_rfc3339();

        if let Some(stats) = existing {
            let update_query = Query::update()
                .table(FileAccessStats::Table)
                .values([
                    (
                        FileAccessStats::AccessCount,
                        (stats.access_count + 1).into(),
                    ),
                    (FileAccessStats::LastAccessedAt, now.clone().into()),
                    (FileAccessStats::UpdatedAt, now.into()),
                ])
                .and_where(Expr::col(FileAccessStats::FileId).eq(file_id))
                .to_string(SqliteQueryBuilder);

            sqlx::query(&update_query).execute(&self.pool).await?;
        } else {
            let insert_query = Query::insert()
                .into_table(FileAccessStats::Table)
                .columns([
                    FileAccessStats::FileId,
                    FileAccessStats::AccessCount,
                    FileAccessStats::LastAccessedAt,
                ])
                .values_panic([file_id.into(), 1i64.into(), now.into()])
                .to_string(SqliteQueryBuilder);

            sqlx::query(&insert_query).execute(&self.pool).await?;
        }

        Ok(())
    }

    pub async fn get_most_accessed_files(
        &self,
        limit: u64,
    ) -> Result<Vec<FileAccessStatsEntry>, TagboxError> {
        let query = Query::select()
            .columns([
                FileAccessStats::FileId,
                FileAccessStats::AccessCount,
                FileAccessStats::LastAccessedAt,
                FileAccessStats::CreatedAt,
                FileAccessStats::UpdatedAt,
            ])
            .from(FileAccessStats::Table)
            .order_by(FileAccessStats::AccessCount, sea_query::Order::Desc)
            .limit(limit)
            .to_string(SqliteQueryBuilder);

        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;

        Ok(rows
            .into_iter()
            .map(|row| FileAccessStatsEntry {
                file_id: row.get(0),
                access_count: row.get(1),
                last_accessed_at: row.get(2),
                created_at: row.get(3),
                updated_at: row.get(4),
            })
            .collect())
    }
}

#[derive(Debug, Clone)]
pub enum FileOperation {
    Create {
        hash: String,
        path: String,
        size: i64,
    },
    Update {
        old_hash: String,
        new_hash: String,
        old_size: i64,
        new_size: i64,
    },
    Move {
        old_path: String,
        new_path: String,
    },
    Delete {
        hash: String,
        path: String,
        size: i64,
    },
    Access,
}

impl FileOperation {
    fn as_str(&self) -> &'static str {
        match self {
            FileOperation::Create { .. } => "create",
            FileOperation::Update { .. } => "update",
            FileOperation::Move { .. } => "move",
            FileOperation::Delete { .. } => "delete",
            FileOperation::Access => "access",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup_test_db() -> (TempDir, SqlitePool) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create the database file
        std::fs::File::create(&db_path).unwrap();

        let pool = SqlitePool::connect(&format!("sqlite:{}", db_path.display()))
            .await
            .unwrap();

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
                reason TEXT
            );

            CREATE TABLE IF NOT EXISTS file_access_stats (
                file_id TEXT PRIMARY KEY,
                access_count INTEGER NOT NULL DEFAULT 0,
                last_accessed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        (temp_dir, pool)
    }

    #[tokio::test]
    async fn test_record_create_operation() {
        let (_temp_dir, pool) = setup_test_db().await;
        let manager = FileHistoryManager::new(pool);

        let file_id = "test-file-id";
        let operation = FileOperation::Create {
            hash: "abc123".to_string(),
            path: "/test/path".to_string(),
            size: 1024,
        };

        let history_id = manager
            .record_file_history(
                file_id,
                operation,
                Some("test-user"),
                Some("Initial import"),
            )
            .await
            .unwrap();

        assert!(!history_id.is_empty());

        let history = manager.get_file_history(file_id, Some(10)).await.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].operation, "create");
        assert_eq!(history[0].new_hash, Some("abc123".to_string()));
        assert_eq!(history[0].changed_by, Some("test-user".to_string()));
    }

    #[tokio::test]
    async fn test_access_stats() {
        let (_temp_dir, pool) = setup_test_db().await;
        let manager = FileHistoryManager::new(pool);

        let file_id = "test-file-id";

        for _ in 0..3 {
            manager
                .record_file_history(file_id, FileOperation::Access, None, None)
                .await
                .unwrap();
        }

        let stats = manager.get_access_stats(file_id).await.unwrap().unwrap();
        assert_eq!(stats.access_count, 3);
    }

    #[tokio::test]
    async fn test_most_accessed_files() {
        let (_temp_dir, pool) = setup_test_db().await;
        let manager = FileHistoryManager::new(pool);

        for i in 0..5 {
            let file_id = format!("file-{}", i);
            for _ in 0..=i {
                manager
                    .record_file_history(&file_id, FileOperation::Access, None, None)
                    .await
                    .unwrap();
            }
        }

        let most_accessed = manager.get_most_accessed_files(3).await.unwrap();
        assert_eq!(most_accessed.len(), 3);
        assert_eq!(most_accessed[0].file_id, "file-4");
        assert_eq!(most_accessed[0].access_count, 5);
    }
}
