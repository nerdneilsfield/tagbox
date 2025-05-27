mod authors;
pub mod config;
mod editor;
pub mod errors;
pub mod history;
mod importer;
mod link;
pub mod metainfo;
pub mod pathgen;
pub mod schema;
mod search;
mod system;
pub mod types;
pub mod utils;
mod validation;

// 导出各个管理器供外部使用
pub use authors::AuthorManager;
pub use editor::Editor;
pub use history::{FileHistoryManager, FileOperation};
pub use importer::Importer;
pub use link::LinkManager;
pub use search::Searcher;
pub use system::{CompatibilityResult, SystemConfigManager};
pub use validation::{FileValidator, ValidationResult, ValidationStatus};

use config::AppConfig;
use errors::Result;
use schema::Database;
use std::path::{Path, PathBuf};
use types::{FileEntry, FileUpdateRequest, ImportMetadata, SearchOptions, SearchResult};

/// 初始化数据库 - Initialize database
pub async fn init_database(path: &Path) -> Result<()> {
    let db = Database::new(path).await?;
    db.migrate().await?;
    Ok(())
}

/// 加载配置
pub async fn load_config(path: &Path) -> Result<AppConfig> {
    AppConfig::from_file(path).await
}

/// 从文件中提取元数据信息
pub async fn extract_metainfo(path: &Path, config: &AppConfig) -> Result<ImportMetadata> {
    let db = Database::new(&config.database.path).await?;
    let _importer = Importer::new(config.clone(), db.pool().clone());

    let metainfo = metainfo::MetaInfoExtractor::new(config.clone());
    metainfo.extract(path).await
}

/// 导入文件到库中
pub async fn import_file(
    path: &Path,
    _metadata: ImportMetadata,
    config: &AppConfig,
) -> Result<FileEntry> {
    let db = Database::new(&config.database.path).await?;
    let importer = Importer::new(config.clone(), db.pool().clone());

    importer.import(path).await
}

// 提取文件元数据并导入数据
pub async fn extract_and_import_file(path: &Path, config: &AppConfig) -> Result<FileEntry> {
    let metadata = extract_metainfo(path, config).await?;
    import_file(path, metadata, config).await
}

/// 批量提取文件元数据并导入数据
///
/// 采用并行提取元数据 + 串行数据库写入的策略来优化性能
/// SQLite 写入会锁定整个数据库，所以数据库操作必须串行执行
pub async fn extract_and_import_files(
    paths: &[&Path],
    config: &AppConfig,
) -> Result<Vec<FileEntry>> {
    use futures::stream::{self, StreamExt};
    use tracing::{info, warn};

    // 第一阶段：并行提取所有文件的元数据
    // 这是 CPU 密集型操作，可以充分利用多核
    info!(
        "Starting parallel metadata extraction for {} files",
        paths.len()
    );

    let metadata_futures = paths.iter().map(|path| {
        let path_clone = path.to_path_buf();
        let config_clone = config.clone();
        async move {
            let path_str = path_clone.to_string_lossy().to_string();
            match extract_metainfo(&path_clone, &config_clone).await {
                Ok(metadata) => Ok((path_clone, metadata)),
                Err(e) => {
                    warn!("Failed to extract metadata from {}: {}", path_str, e);
                    Err(e)
                }
            }
        }
    });

    // 使用 buffer_unordered 限制并发数，避免打开太多文件
    let max_concurrent = num_cpus::get().min(8); // 最多 8 个并发任务
    let metadata_results: Vec<_> = stream::iter(metadata_futures)
        .buffer_unordered(max_concurrent)
        .collect()
        .await;

    info!(
        "Metadata extraction completed, processing {} results",
        metadata_results.len()
    );

    // 收集成功提取元数据的文件
    let mut metadata_pairs = Vec::new();
    let mut extraction_errors = Vec::new();

    for (i, result) in metadata_results.into_iter().enumerate() {
        match result {
            Ok(pair) => {
                info!(
                    "Metadata extraction success for file {}: {}",
                    i,
                    pair.0.display()
                );
                metadata_pairs.push(pair);
            }
            Err(e) => {
                warn!("Metadata extraction failed for file {}: {:?}", i, e);
                extraction_errors.push(e);
            }
        }
    }

    if !extraction_errors.is_empty() {
        warn!(
            "Metadata extraction failed for {} files out of {}",
            extraction_errors.len(),
            paths.len()
        );
    }

    info!(
        "Successfully extracted metadata for {} files",
        metadata_pairs.len()
    );

    // 第二阶段：串行导入到数据库
    // SQLite 不支持并发写入，必须一个一个导入
    info!("Starting sequential database import");

    // 创建一个共享的数据库连接池，避免重复创建
    let db = Database::new(&config.database.path).await?;
    let importer = Importer::new(config.clone(), db.pool().clone());

    let mut entries = Vec::new();
    let mut import_errors = Vec::new();

    for (path, metadata) in metadata_pairs {
        let path_str = path.to_string_lossy().to_string();

        // 使用已经创建的 importer 实例，避免重复创建数据库连接
        info!("Attempting to import: {}", path_str);
        match importer.import_with_metadata(&path, metadata).await {
            Ok(entry) => {
                info!("Successfully imported: {} (ID: {})", path_str, entry.id);
                entries.push(entry);
            }
            Err(e) => {
                warn!("Failed to import {}: {:?}", path_str, e);
                import_errors.push((path_str, e));
            }
        }
    }

    // 报告结果
    info!(
        "Import completed: {} succeeded, {} failed (extraction: {}, import: {})",
        entries.len(),
        extraction_errors.len() + import_errors.len(),
        extraction_errors.len(),
        import_errors.len()
    );

    // 如果所有文件都失败了，返回错误
    if entries.is_empty() && !paths.is_empty() {
        return Err(errors::TagboxError::ImportError(
            "All files failed to import".to_string(),
        ));
    }

    Ok(entries)
}

/// 简单文件搜索
pub async fn search_files(query: &str, config: &AppConfig) -> Result<Vec<FileEntry>> {
    let db = Database::new(&config.database.path).await?;
    let searcher = Searcher::new(config.clone(), db.pool().clone()).await;

    searcher.search(query).await
}

/// 高级文件搜索
pub async fn search_files_advanced(
    query: &str,
    options: Option<SearchOptions>,
    config: &AppConfig,
) -> Result<SearchResult> {
    let db = Database::new(&config.database.path).await?;
    let searcher = Searcher::new(config.clone(), db.pool().clone()).await;

    searcher.search_advanced(query, options).await
}

/// 模糊文件搜索
pub async fn fuzzy_search_files(
    text: &str,
    options: Option<SearchOptions>,
    config: &AppConfig,
) -> Result<SearchResult> {
    let db = Database::new(&config.database.path).await?;
    let searcher = Searcher::new(config.clone(), db.pool().clone()).await;

    searcher.fuzzy_search(text, options).await
}

/// 重建全文搜索索引
pub async fn rebuild_search_index(config: &AppConfig) -> Result<()> {
    let db = Database::new(&config.database.path).await?;
    let searcher = Searcher::new(config.clone(), db.pool().clone()).await;

    searcher.init_fts().await?;
    searcher.rebuild_fts_index().await
}

/// 获取文件路径
pub async fn get_file_path(file_id: &str, config: &AppConfig) -> Result<PathBuf> {
    let db = Database::new(&config.database.path).await?;
    let editor = Editor::new(db.pool().clone());

    editor.get_file_path(file_id).await
}

/// 获取文件信息
pub async fn get_file(file_id: &str, config: &AppConfig) -> Result<FileEntry> {
    let db = Database::new(&config.database.path).await?;
    let editor = Editor::new(db.pool().clone());

    editor.get_file(file_id).await
}

/// 编辑文件信息
pub async fn edit_file(file_id: &str, update: FileUpdateRequest, config: &AppConfig) -> Result<()> {
    let db = Database::new(&config.database.path).await?;
    let editor = Editor::new(db.pool().clone());

    editor.update_file(file_id, update).await
}

/// 建立文件之间的关联
pub async fn link_files(
    file_id_a: &str,
    file_id_b: &str,
    relation: Option<String>,
    config: &AppConfig,
) -> Result<()> {
    let db = Database::new(&config.database.path).await?;
    let link_manager = LinkManager::new(db.pool().clone());

    link_manager
        .create_link(file_id_a, file_id_b, relation)
        .await
}

/// 解除文件之间的关联
pub async fn unlink_files(file_id_a: &str, file_id_b: &str, config: &AppConfig) -> Result<()> {
    let db = Database::new(&config.database.path).await?;
    let link_manager = LinkManager::new(db.pool().clone());

    link_manager.remove_link(file_id_a, file_id_b).await
}

/// 验证单个文件的完整性
pub async fn validate_file(path: &Path, config: &AppConfig) -> Result<ValidationResult> {
    let db = Database::new(&config.database.path).await?;
    let validator = FileValidator::new(db.pool().clone(), config.clone());

    validator.validate_single_file(path).await
}

/// 验证目录中的文件完整性
pub async fn validate_files_in_path(
    path: &Path,
    recursive: bool,
    config: &AppConfig,
) -> Result<Vec<ValidationResult>> {
    let db = Database::new(&config.database.path).await?;
    let validator = FileValidator::new(db.pool().clone(), config.clone());

    validator.validate_files_in_path(path, recursive).await
}

/// 更新文件哈希值
pub async fn update_file_hash(
    file_id: &str,
    reason: &str,
    config: &AppConfig,
) -> Result<FileEntry> {
    let db = Database::new(&config.database.path).await?;
    let validator = FileValidator::new(db.pool().clone(), config.clone());

    validator.update_file_hash(file_id, reason).await
}

/// 检查配置兼容性
pub async fn check_config_compatibility(config: &AppConfig) -> Result<CompatibilityResult> {
    let db = Database::new(&config.database.path).await?;
    let system_manager = SystemConfigManager::new(db.pool().clone());

    system_manager.check_config_compatibility(config).await
}

/// 记录文件历史
pub async fn record_file_history(
    file_id: &str,
    operation: FileOperation,
    changed_by: Option<&str>,
    reason: Option<&str>,
    config: &AppConfig,
) -> Result<String> {
    let db = Database::new(&config.database.path).await?;
    let history_manager = FileHistoryManager::new(db.pool().clone());

    history_manager
        .record_file_history(file_id, operation, changed_by, reason)
        .await
}

/// 获取文件历史记录
pub async fn get_file_history(
    file_id: &str,
    limit: Option<u64>,
    config: &AppConfig,
) -> Result<Vec<history::FileHistoryEntry>> {
    let db = Database::new(&config.database.path).await?;
    let history_manager = FileHistoryManager::new(db.pool().clone());

    history_manager.get_file_history(file_id, limit).await
}

/// 获取文件访问统计
pub async fn get_file_access_stats(
    file_id: &str,
    config: &AppConfig,
) -> Result<Option<history::FileAccessStatsEntry>> {
    let db = Database::new(&config.database.path).await?;
    let history_manager = FileHistoryManager::new(db.pool().clone());

    history_manager.get_access_stats(file_id).await
}

/// 获取访问最多的文件
pub async fn get_most_accessed_files(
    limit: u64,
    config: &AppConfig,
) -> Result<Vec<history::FileAccessStatsEntry>> {
    let db = Database::new(&config.database.path).await?;
    let history_manager = FileHistoryManager::new(db.pool().clone());

    history_manager.get_most_accessed_files(limit).await
}
