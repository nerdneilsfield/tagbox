use std::sync::mpsc::Sender;
use std::future::Future;
use tagbox_core::config::AppConfig;
use tagbox_core::types::SearchOptions;
use crate::state::AppEvent;
use tracing::{info, warn, error};

pub struct AsyncBridge {
    pub runtime: tokio::runtime::Runtime,
    event_sender: Sender<AppEvent>,
}

impl AsyncBridge {
    pub fn new() -> Self {
        let runtime = tokio::runtime::Runtime::new()
            .expect("Failed to create async runtime");
        
        // 这里需要从外部传入 event_sender，暂时使用占位符
        // 实际使用时会通过构造函数传入
        let (event_sender, _) = std::sync::mpsc::channel();
        
        Self {
            runtime,
            event_sender,
        }
    }
    
    pub fn with_sender(event_sender: Sender<AppEvent>) -> Self {
        let runtime = tokio::runtime::Runtime::new()
            .expect("Failed to create async runtime");
        
        Self {
            runtime,
            event_sender,
        }
    }
    
    pub fn spawn_task<F, R>(&self, future: F) 
    where
        F: Future<Output = Result<R, Box<dyn std::error::Error + Send + Sync>>> + Send + 'static,
        R: Send + 'static,
    {
        let sender = self.event_sender.clone();
        self.runtime.spawn(async move {
            match future.await {
                Ok(_result) => {
                    // 根据结果类型发送相应事件
                    // 这里需要根据具体情况处理
                }
                Err(e) => {
                    let _ = sender.send(AppEvent::Error(e.to_string()));
                }
            }
        });
    }
    
    pub fn spawn_search(&self, query: String, config: AppConfig) {
        let sender = self.event_sender.clone();
        self.runtime.spawn(async move {
            // 发送加载开始事件
            let _ = sender.send(AppEvent::LoadingStart);
            
            match tagbox_core::search_files_advanced(&query, None, &config).await {
                Ok(result) => {
                    let _ = sender.send(AppEvent::SearchResults(result));
                }
                Err(e) => {
                    let _ = sender.send(AppEvent::Error(format!("Search failed: {}", e)));
                }
            }
            
            // 发送加载结束事件
            let _ = sender.send(AppEvent::LoadingEnd);
        });
    }
    
    pub fn spawn_load_file(&self, file_id: String, config: AppConfig) {
        let sender = self.event_sender.clone();
        self.runtime.spawn(async move {
            info!("Loading file details for: {}", file_id);
            match tagbox_core::get_file(&file_id, &config).await {
                Ok(file) => {
                    info!("Successfully loaded file: {}", file.title);
                    let _ = sender.send(AppEvent::FileLoaded(file));
                }
                Err(e) => {
                    error!("Failed to load file {}: {}", file_id, e);
                    let _ = sender.send(AppEvent::Error(format!("Load file failed: {}", e)));
                }
            }
        });
    }
    
    pub fn spawn_import_file(&self, file_path: std::path::PathBuf, config: AppConfig) {
        let sender = self.event_sender.clone();
        self.runtime.spawn(async move {
            info!("Starting file import: {}", file_path.display());
            let _ = sender.send(AppEvent::LoadingStart);
            
            match tagbox_core::extract_and_import_file(&file_path, &config).await {
                Ok(file_entry) => {
                    info!("Successfully imported file: {} -> {}", file_path.display(), file_entry.id);
                    let _ = sender.send(AppEvent::FileImported(file_entry));
                    let _ = sender.send(AppEvent::RefreshView);
                }
                Err(e) => {
                    error!("Failed to import file {}: {}", file_path.display(), e);
                    let _ = sender.send(AppEvent::Error(format!("Import failed: {}", e)));
                }
            }
            
            let _ = sender.send(AppEvent::LoadingEnd);
        });
    }

    pub fn spawn_advanced_search(&self, options: SearchOptions, config: AppConfig) {
        let sender = self.event_sender.clone();
        self.runtime.spawn(async move {
            info!("Starting advanced search with options: {:?}", options);
            let _ = sender.send(AppEvent::LoadingStart);
            
            // 对于高级搜索，我们使用通配符查询来获取所有结果
            // 然后通过 SearchOptions 中的 sort_by 和其他选项来过滤
            let query = "*".to_string(); // 搜索所有文件
            
            match tagbox_core::search_files_advanced(&query, Some(options), &config).await {
                Ok(result) => {
                    info!("Advanced search completed: {} results", result.entries.len());
                    let _ = sender.send(AppEvent::SearchResults(result));
                }
                Err(e) => {
                    error!("Advanced search failed: {}", e);
                    let _ = sender.send(AppEvent::Error(format!("Advanced search failed: {}", e)));
                }
            }
            
            let _ = sender.send(AppEvent::LoadingEnd);
        });
    }
    
    pub fn spawn_category_search(&self, category_path: String, config: AppConfig) {
        let sender = self.event_sender.clone();
        self.runtime.spawn(async move {
            info!("Searching files in category: {}", category_path);
            let _ = sender.send(AppEvent::LoadingStart);
            
            // 构建分类搜索查询
            let query = if category_path.is_empty() || category_path == "All Files" {
                "*".to_string() // 显示所有文件
            } else {
                format!("category:{}", category_path)
            };
            
            match tagbox_core::search_files_advanced(&query, None, &config).await {
                Ok(result) => {
                    info!("Category search completed: {} results", result.entries.len());
                    let _ = sender.send(AppEvent::SearchResults(result));
                }
                Err(e) => {
                    error!("Category search failed: {}", e);
                    let _ = sender.send(AppEvent::Error(format!("Category search failed: {}", e)));
                }
            }
            
            let _ = sender.send(AppEvent::LoadingEnd);
        });
    }

    pub fn spawn_load_all_files(&self, config: AppConfig) {
        let sender = self.event_sender.clone();
        self.runtime.spawn(async move {
            info!("Loading all files");
            info!("Database path: {}", config.database.path.display());
            let _ = sender.send(AppEvent::LoadingStart);
            
            // 使用空查询搜索所有文件
            match tagbox_core::search_files_advanced("*", None, &config).await {
                Ok(result) => {
                    info!("Loaded {} files", result.entries.len());
                    for entry in &result.entries {
                        info!("File: {} - {}", entry.id, entry.title);
                    }
                    let _ = sender.send(AppEvent::SearchResults(result));
                }
                Err(e) => {
                    error!("Failed to load files: {}", e);
                    error!("Error details: {:?}", e);
                    let _ = sender.send(AppEvent::Error(format!("Failed to load files: {}", e)));
                }
            }
            
            let _ = sender.send(AppEvent::LoadingEnd);
        });
    }

    pub fn spawn_batch_import(&self, file_paths: Vec<std::path::PathBuf>, config: AppConfig) {
        let sender = self.event_sender.clone();
        let total_files = file_paths.len();
        
        self.runtime.spawn(async move {
            info!("Starting batch import of {} files", total_files);
            let _ = sender.send(AppEvent::LoadingStart);
            
            let mut successful_imports = 0;
            let mut failed_imports = 0;
            
            for (index, file_path) in file_paths.iter().enumerate() {
                info!("Importing file {}/{}: {}", index + 1, total_files, file_path.display());
                
                match tagbox_core::extract_and_import_file(file_path, &config).await {
                    Ok(file_entry) => {
                        successful_imports += 1;
                        info!("Successfully imported: {} -> {}", file_path.display(), file_entry.id);
                        let _ = sender.send(AppEvent::FileImported(file_entry));
                    }
                    Err(e) => {
                        failed_imports += 1;
                        warn!("Failed to import {}: {}", file_path.display(), e);
                        let _ = sender.send(AppEvent::Error(format!("Failed to import {}: {}", file_path.display(), e)));
                    }
                }
            }
            
            info!("Batch import completed: {} successful, {} failed", successful_imports, failed_imports);
            let _ = sender.send(AppEvent::RefreshView);
            let _ = sender.send(AppEvent::LoadingEnd);
        });
    }
}