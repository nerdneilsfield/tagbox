mod authors;
pub mod config;
mod editor;
pub mod errors;
mod importer;
mod link;
pub mod metainfo;
pub mod pathgen;
mod schema;
mod search;
pub mod types;
pub mod utils;

use config::AppConfig;
use editor::Editor;
use errors::Result;
use importer::Importer;
use link::LinkManager;
use schema::Database;
use search::Searcher;
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
