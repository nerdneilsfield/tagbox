use crate::{config::AppConfig, errors::TagboxError};
use sea_query::{Expr, Iden, Query, SqliteQueryBuilder};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;

#[derive(Iden)]
enum SystemConfig {
    Table,
    Key,
    Value,
    Description,
    CreatedAt,
    UpdatedAt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfigEntry {
    pub key: String,
    pub value: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub struct SystemConfigManager {
    pool: SqlitePool,
}

impl SystemConfigManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn set_config(
        &self,
        key: &str,
        value: &str,
        description: Option<&str>,
    ) -> Result<(), TagboxError> {
        let existing = self.get_config(key).await?;

        if existing.is_some() {
            let update_query = Query::update()
                .table(SystemConfig::Table)
                .values([
                    (SystemConfig::Value, value.into()),
                    (SystemConfig::UpdatedAt, "CURRENT_TIMESTAMP".into()),
                ])
                .and_where(Expr::col(SystemConfig::Key).eq(key))
                .to_string(SqliteQueryBuilder);

            sqlx::query(&update_query).execute(&self.pool).await?;
        } else {
            let mut columns = vec![SystemConfig::Key, SystemConfig::Value];
            let mut values: Vec<sea_query::SimpleExpr> = vec![key.into(), value.into()];

            if let Some(desc) = description {
                columns.push(SystemConfig::Description);
                values.push(desc.into());
            }

            let insert_query = Query::insert()
                .into_table(SystemConfig::Table)
                .columns(columns)
                .values_panic(values)
                .to_string(SqliteQueryBuilder);

            sqlx::query(&insert_query).execute(&self.pool).await?;
        }

        Ok(())
    }

    pub async fn get_config(&self, key: &str) -> Result<Option<SystemConfigEntry>, TagboxError> {
        let query = Query::select()
            .columns([
                SystemConfig::Key,
                SystemConfig::Value,
                SystemConfig::Description,
                SystemConfig::CreatedAt,
                SystemConfig::UpdatedAt,
            ])
            .from(SystemConfig::Table)
            .and_where(Expr::col(SystemConfig::Key).eq(key))
            .to_string(SqliteQueryBuilder);

        let row = sqlx::query(&query).fetch_optional(&self.pool).await?;

        Ok(row.map(|row| SystemConfigEntry {
            key: row.get(0),
            value: row.get(1),
            description: row.get(2),
            created_at: row.get(3),
            updated_at: row.get(4),
        }))
    }

    pub async fn get_all_configs(&self) -> Result<Vec<SystemConfigEntry>, TagboxError> {
        let query = Query::select()
            .columns([
                SystemConfig::Key,
                SystemConfig::Value,
                SystemConfig::Description,
                SystemConfig::CreatedAt,
                SystemConfig::UpdatedAt,
            ])
            .from(SystemConfig::Table)
            .to_string(SqliteQueryBuilder);

        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;

        Ok(rows
            .into_iter()
            .map(|row| SystemConfigEntry {
                key: row.get(0),
                value: row.get(1),
                description: row.get(2),
                created_at: row.get(3),
                updated_at: row.get(4),
            })
            .collect())
    }

    pub async fn delete_config(&self, key: &str) -> Result<(), TagboxError> {
        let query = Query::delete()
            .from_table(SystemConfig::Table)
            .and_where(Expr::col(SystemConfig::Key).eq(key))
            .to_string(SqliteQueryBuilder);

        sqlx::query(&query).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn check_config_compatibility(
        &self,
        config: &AppConfig,
    ) -> Result<CompatibilityResult, TagboxError> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let stored_configs: HashMap<String, String> = self
            .get_all_configs()
            .await?
            .into_iter()
            .map(|entry| (entry.key, entry.value))
            .collect();

        if let Some(stored_hash_algo) = stored_configs.get("hash_algorithm") {
            if stored_hash_algo != &config.hash.algorithm {
                warnings.push(format!(
                    "Hash algorithm mismatch: database uses '{}', config uses '{}'",
                    stored_hash_algo, config.hash.algorithm
                ));
            }
        } else {
            self.set_config(
                "hash_algorithm",
                &config.hash.algorithm,
                Some("Hash algorithm used for file integrity"),
            )
            .await?;
        }

        if let Some(stored_data_dir) = stored_configs.get("data_directory") {
            let config_data_dir = config.import.paths.storage_dir.to_string_lossy();
            if stored_data_dir != &config_data_dir {
                errors.push(format!(
                    "Data directory mismatch: database uses '{}', config uses '{}'",
                    stored_data_dir, config_data_dir
                ));
            }
        } else {
            self.set_config(
                "data_directory",
                &config.import.paths.storage_dir.to_string_lossy(),
                Some("Base data directory for file storage"),
            )
            .await?;
        }

        if let Some(stored_db_version) = stored_configs.get("database_version") {
            let current_version = self.get_current_db_version();
            if stored_db_version != &current_version {
                warnings.push(format!(
                    "Database version mismatch: stored '{}', current '{}'",
                    stored_db_version, current_version
                ));
            }
        } else {
            self.set_config(
                "database_version",
                &self.get_current_db_version(),
                Some("Database schema version"),
            )
            .await?;
        }

        Ok(CompatibilityResult {
            is_compatible: errors.is_empty(),
            warnings,
            errors,
        })
    }

    fn get_current_db_version(&self) -> String {
        "1.0.0".to_string()
    }
}

#[derive(Debug, Clone)]
pub struct CompatibilityResult {
    pub is_compatible: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
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
            CREATE TABLE IF NOT EXISTS system_config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                description TEXT,
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
    async fn test_set_and_get_config() {
        let (_temp_dir, pool) = setup_test_db().await;
        let manager = SystemConfigManager::new(pool);

        manager
            .set_config("test_key", "test_value", Some("Test description"))
            .await
            .unwrap();

        let config = manager.get_config("test_key").await.unwrap().unwrap();
        assert_eq!(config.key, "test_key");
        assert_eq!(config.value, "test_value");
        assert_eq!(config.description, Some("Test description".to_string()));
    }

    #[tokio::test]
    async fn test_update_config() {
        let (_temp_dir, pool) = setup_test_db().await;
        let manager = SystemConfigManager::new(pool);

        manager
            .set_config("test_key", "value1", None)
            .await
            .unwrap();

        manager
            .set_config("test_key", "value2", None)
            .await
            .unwrap();

        let config = manager.get_config("test_key").await.unwrap().unwrap();
        assert_eq!(config.value, "value2");
    }

    #[tokio::test]
    async fn test_delete_config() {
        let (_temp_dir, pool) = setup_test_db().await;
        let manager = SystemConfigManager::new(pool);

        manager
            .set_config("test_key", "test_value", None)
            .await
            .unwrap();

        manager.delete_config("test_key").await.unwrap();

        let config = manager.get_config("test_key").await.unwrap();
        assert!(config.is_none());
    }

    #[tokio::test]
    async fn test_check_config_compatibility() {
        let (temp_dir, pool) = setup_test_db().await;
        let manager = SystemConfigManager::new(pool.clone());

        let mut config = AppConfig::default();
        config.import.paths.storage_dir = temp_dir.path().to_path_buf();
        config.hash.algorithm = "sha256".to_string();

        let result = manager.check_config_compatibility(&config).await.unwrap();
        assert!(result.is_compatible);
        assert!(result.errors.is_empty());

        manager
            .set_config("data_directory", "/different/path", None)
            .await
            .unwrap();

        let result = manager.check_config_compatibility(&config).await.unwrap();
        assert!(!result.is_compatible);
        assert_eq!(result.errors.len(), 1);
    }
}
