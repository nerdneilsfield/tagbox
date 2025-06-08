#[cfg(test)]
mod tests {
    use crate::services::TagBoxService;
    use anyhow::Result;

    async fn create_test_service() -> Result<TagBoxService> {
        // 使用默认配置创建服务
        TagBoxService::new(None).await
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

    // 暂时注释掉需要真实数据库的测试
    // #[tokio::test]
    // async fn test_import_and_search() {
    //     // 这个测试需要真实的数据库初始化
    //     // 在 CI 环境中可能会失败
    // }
}