use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tagbox_core::{
    config::AppConfig,
    schema::Database,
    types::{FileEntry, ImportMetadata, SearchOptions, SearchResult},
    FileOperation, LinkManager,
};
use tokio::sync::Mutex;

/// TagBox 服务层，封装所有 tagbox-core 的 API 调用
pub struct TagBoxService {
    config: AppConfig,
    config_path: Option<PathBuf>,
    db: Arc<Mutex<Database>>,
}

impl TagBoxService {
    /// 创建新的 TagBox 服务实例
    pub async fn new(config_path: Option<&str>) -> Result<Self> {
        // 参考 CLI 的配置加载方式
        let config = if let Some(path) = config_path {
            AppConfig::from_file(Path::new(path)).await?
        } else {
            // 查找默认配置文件位置
            let default_paths = [
                PathBuf::from("config.toml"),
                dirs::config_dir()
                    .map(|d| d.join("tagbox").join("config.toml"))
                    .unwrap_or_default(),
            ];
            
            let mut loaded_config = None;
            for path in &default_paths {
                if path.exists() {
                    match AppConfig::from_file(path).await {
                        Ok(cfg) => {
                            loaded_config = Some(cfg);
                            break;
                        }
                        Err(_) => continue,
                    }
                }
            }
            
            loaded_config.unwrap_or_else(AppConfig::default)
        };

        // 创建数据库连接
        let db = Database::new(&config.database.path).await?;
        
        let service_config_path = config_path.map(|p| PathBuf::from(p));
        
        Ok(Self {
            config,
            config_path: service_config_path,
            db: Arc::new(Mutex::new(db)),
        })
    }

    /// 获取配置
    pub fn config(&self) -> &AppConfig {
        &self.config
    }
    
    /// 获取配置文件路径
    pub fn config_path(&self) -> Option<PathBuf> {
        self.config_path.clone()
    }
    
    /// 重新加载配置
    pub async fn reload_config(&mut self) -> Result<()> {
        if let Some(path) = &self.config_path {
            self.config = AppConfig::from_file(path).await?;
        }
        Ok(())
    }

    /// 搜索文件 - 参考 commands/search.rs
    pub async fn search(&self, query: &str, options: Option<SearchOptions>) -> Result<SearchResult> {
        let options = options.unwrap_or(SearchOptions {
            offset: 0,
            limit: 50,
            sort_by: None,
            sort_direction: None,
            include_deleted: false,
        });

        tagbox_core::search_files_advanced(query, Some(options), &self.config).await
            .map_err(|e| anyhow::anyhow!("Search failed: {}", e))
    }

    /// 模糊搜索
    pub async fn fuzzy_search(&self, text: &str, options: Option<SearchOptions>) -> Result<SearchResult> {
        tagbox_core::fuzzy_search_files(text, options, &self.config).await
            .map_err(|e| anyhow::anyhow!("Fuzzy search failed: {}", e))
    }

    /// 列出所有文件 - 参考 commands/list.rs
    pub async fn list_all(&self, limit: Option<usize>) -> Result<SearchResult> {
        self.search("*", Some(SearchOptions {
            offset: 0,
            limit: limit.unwrap_or(100),
            sort_by: Some("imported_at".to_string()),
            sort_direction: Some("desc".to_string()),
            include_deleted: false,
        })).await
    }

    /// 提取文件元数据 - 参考 commands/import.rs
    pub async fn extract_metadata(&self, path: &Path) -> Result<ImportMetadata> {
        tagbox_core::extract_metainfo(path, &self.config).await
            .map_err(|e| anyhow::anyhow!("Metadata extraction failed: {}", e))
    }

    /// 导入单个文件
    pub async fn import_file(&self, path: &Path, metadata: Option<ImportMetadata>) -> Result<FileEntry> {
        let metadata = if let Some(m) = metadata {
            m
        } else {
            self.extract_metadata(path).await?
        };

        tagbox_core::import_file(path, metadata, &self.config).await
            .map_err(|e| anyhow::anyhow!("Import failed: {}", e))
    }

    /// 批量导入文件
    pub async fn import_files(&self, paths: Vec<&Path>) -> Result<Vec<FileEntry>> {
        tagbox_core::extract_and_import_files(&paths, &self.config).await
            .map_err(|e| anyhow::anyhow!("Batch import failed: {}", e))
    }

    /// 获取文件信息
    pub async fn get_file(&self, file_id: &str) -> Result<FileEntry> {
        tagbox_core::get_file(file_id, &self.config).await
            .map_err(|e| anyhow::anyhow!("Get file failed: {}", e))
    }

    /// 更新文件元数据 - 参考 commands/edit.rs
    pub async fn update_file(&self, file_id: &str, metadata: ImportMetadata) -> Result<()> {
        use tagbox_core::types::FileUpdateRequest;
        
        let update_request = FileUpdateRequest {
            title: if metadata.title.is_empty() { None } else { Some(metadata.title) },
            authors: if metadata.authors.is_empty() { None } else { Some(metadata.authors) },
            year: metadata.year,
            publisher: metadata.publisher,
            source: metadata.source,
            category1: if metadata.category1.is_empty() { None } else { Some(metadata.category1) },
            category2: metadata.category2,
            category3: metadata.category3,
            tags: if metadata.tags.is_empty() { None } else { Some(metadata.tags) },
            summary: metadata.summary,
            full_text: metadata.full_text,
            is_deleted: None,
            file_metadata: metadata.file_metadata,
            type_metadata: metadata.type_metadata,
        };

        tagbox_core::edit_file(file_id, update_request, &self.config).await
            .map_err(|e| anyhow::anyhow!("Update file failed: {}", e))
    }

    /// 删除文件（软删除）
    pub async fn delete_file(&self, file_id: &str) -> Result<()> {
        use tagbox_core::types::FileUpdateRequest;
        
        let update_request = FileUpdateRequest {
            title: None,
            authors: None,
            year: None,
            publisher: None,
            source: None,
            category1: None,
            category2: None,
            category3: None,
            tags: None,
            summary: None,
            full_text: None,
            is_deleted: Some(true),
            file_metadata: None,
            type_metadata: None,
        };

        tagbox_core::edit_file(file_id, update_request, &self.config).await
            .map_err(|e| anyhow::anyhow!("Delete file failed: {}", e))
    }

    /// 获取分类列表
    pub async fn get_categories(&self) -> Result<Vec<String>> {
        // TODO: 实现分类查询
        // 暂时返回示例分类
        Ok(vec![
            "技术/编程/Rust".to_string(),
            "技术/编程/Python".to_string(),
            "技术/数据库".to_string(),
            "文学/小说".to_string(),
            "文学/诗歌".to_string(),
        ])
    }

    /// 获取作者列表
    pub async fn get_authors(&self) -> Result<Vec<String>> {
        // TODO: 实现真实的作者查询
        // 暂时返回空列表
        Ok(vec![])
    }

    /// 创建文件关联
    pub async fn link_files(&self, file_id_a: &str, file_id_b: &str, relation: Option<String>) -> Result<()> {
        tagbox_core::link_files(file_id_a, file_id_b, relation, &self.config).await
            .map_err(|e| anyhow::anyhow!("Link files failed: {}", e))
    }

    /// 解除文件关联
    pub async fn unlink_files(&self, file_id_a: &str, file_id_b: &str) -> Result<()> {
        tagbox_core::unlink_files(file_id_a, file_id_b, &self.config).await
            .map_err(|e| anyhow::anyhow!("Unlink files failed: {}", e))
    }

    /// 获取文件关联
    pub async fn get_linked_files(&self, file_id: &str) -> Result<Vec<FileEntry>> {
        let db = self.db.lock().await;
        let link_manager = LinkManager::new(db.pool().clone());
        
        let links = link_manager.get_links_for_file(file_id).await
            .map_err(|e| anyhow::anyhow!("Get links failed: {}", e))?;

        // 获取关联文件的详细信息
        let mut linked_files = Vec::new();
        for (source_id, target_id, _relation) in links {
            let other_id = if source_id == file_id { target_id } else { source_id };
            if let Ok(file) = self.get_file(&other_id).await {
                linked_files.push(file);
            }
        }

        Ok(linked_files)
    }

    /// 记录文件历史
    pub async fn record_file_history(
        &self,
        file_id: &str,
        operation: FileOperation,
        reason: Option<&str>,
    ) -> Result<()> {
        tagbox_core::record_file_history(
            file_id,
            operation,
            Some("GUI User"),
            reason,
            &self.config,
        ).await
        .map_err(|e| anyhow::anyhow!("Record history failed: {}", e))?;
        
        Ok(())
    }

    /// 获取文件历史
    pub async fn get_file_history(&self, file_id: &str, limit: Option<u64>) -> Result<Vec<tagbox_core::history::FileHistoryEntry>> {
        tagbox_core::get_file_history(file_id, limit, &self.config).await
            .map_err(|e| anyhow::anyhow!("Get history failed: {}", e))
    }

    /// 重建搜索索引
    pub async fn rebuild_search_index(&self) -> Result<()> {
        tagbox_core::rebuild_search_index(&self.config).await
            .map_err(|e| anyhow::anyhow!("Rebuild index failed: {}", e))
    }

    /// 检查配置兼容性
    pub async fn check_compatibility(&self) -> Result<tagbox_core::CompatibilityResult> {
        tagbox_core::check_config_compatibility(&self.config).await
            .map_err(|e| anyhow::anyhow!("Compatibility check failed: {}", e))
    }
}

#[cfg(test)]
#[path = "tagbox_service_test.rs"]
mod tests;