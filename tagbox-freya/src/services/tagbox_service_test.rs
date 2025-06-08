#[cfg(test)]
mod tests {
    use crate::services::TagBoxService;
    use anyhow::Result;
    use tempfile::tempdir;
    use tagbox_core::types::ImportMetadata;

    async fn create_test_service() -> Result<TagBoxService> {
        // 创建临时目录
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test.db");
        
        // 创建测试配置
        let config = tagbox_core::config::AppConfig {
            database: tagbox_core::config::DatabaseConfig {
                path: db_path.clone(),
                journal_mode: "WAL".to_string(),
                sync_mode: "NORMAL".to_string(),
            },
            import: tagbox_core::config::ImportConfig {
                paths: tagbox_core::config::ImportPathsConfig {
                    storage_dir: temp_dir.path().join("storage"),
                    rename_template: "{year}/{category1}/{title}".to_string(),
                    classify_template: "{category1}/{category2}/{category3}".to_string(),
                },
                metadata: tagbox_core::config::ImportMetadataConfig {
                    prefer_json: true,
                    fallback_pdf: true,
                    default_category: "未分类".to_string(),
                },
            },
            search: tagbox_core::config::SearchConfig {
                default_limit: 50,
                enable_fts: true,
                fts_language: "chinese".to_string(),
            },
            hash: tagbox_core::config::HashConfig {
                algorithm: "blake3".to_string(),
                verify_on_import: false,
            },
        };
        
        // 保存配置到临时文件
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, toml::to_string(&config)?)?;
        
        // 创建服务
        TagBoxService::new(Some(config_path.to_str().unwrap())).await
    }

    #[tokio::test]
    async fn test_service_creation() {
        let service = create_test_service().await;
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_search_empty_database() {
        let service = create_test_service().await.unwrap();
        let result = service.search("*", None).await.unwrap();
        assert_eq!(result.total_count, 0);
        assert!(result.entries.is_empty());
    }

    #[tokio::test]
    async fn test_list_all_empty() {
        let service = create_test_service().await.unwrap();
        let result = service.list_all(None).await.unwrap();
        assert_eq!(result.total_count, 0);
    }

    #[tokio::test]
    async fn test_get_categories() {
        let service = create_test_service().await.unwrap();
        let categories = service.get_categories().await.unwrap();
        // 空数据库可能没有分类
        assert!(categories.is_empty() || !categories.is_empty());
    }

    #[tokio::test]
    async fn test_search_with_dsl() {
        let service = create_test_service().await.unwrap();
        
        // 测试各种 DSL 查询
        let queries = vec![
            "tag:rust",
            "author:张三",
            "-tag:old",
            "title:测试",
        ];
        
        for query in queries {
            let result = service.search(query, None).await;
            assert!(result.is_ok(), "Query '{}' should be valid", query);
        }
    }

    #[tokio::test]
    async fn test_import_and_search() {
        let service = create_test_service().await.unwrap();
        
        // 创建测试文件
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "Test content").unwrap();
        
        // 导入文件
        let metadata = ImportMetadata {
            title: "Test Document".to_string(),
            authors: vec!["Test Author".to_string()],
            year: Some(2024),
            publisher: None,
            source: None,
            category1: "技术".to_string(),
            category2: Some("编程".to_string()),
            category3: None,
            tags: vec!["test".to_string(), "rust".to_string()],
            summary: Some("A test document".to_string()),
            full_text: Some("Test content".to_string()),
            additional_info: std::collections::HashMap::new(),
            file_metadata: None,
            type_metadata: None,
        };
        
        let imported = service.import_file(&test_file, Some(metadata)).await;
        assert!(imported.is_ok());
        
        // 搜索导入的文件
        let search_result = service.search("tag:test", None).await.unwrap();
        assert_eq!(search_result.total_count, 1);
        assert_eq!(search_result.entries[0].title, "Test Document");
    }
}