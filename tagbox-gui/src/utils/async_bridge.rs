use std::sync::mpsc::Sender;
use std::future::Future;
use tagbox_core::config::AppConfig;
use crate::state::AppEvent;

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
            match tagbox_core::get_file(&file_id, &config).await {
                Ok(_file) => {
                    // 这里需要发送文件加载成功的事件
                    // 暂时使用 FileSelected 事件
                    let _ = sender.send(AppEvent::FileSelected(file_id));
                }
                Err(e) => {
                    let _ = sender.send(AppEvent::Error(format!("Load file failed: {}", e)));
                }
            }
        });
    }
    
    pub fn spawn_import_file(&self, file_path: std::path::PathBuf, config: AppConfig) {
        let sender = self.event_sender.clone();
        self.runtime.spawn(async move {
            let _ = sender.send(AppEvent::LoadingStart);
            
            match tagbox_core::extract_and_import_file(&file_path, &config).await {
                Ok(_file_entry) => {
                    // 导入成功，刷新视图
                    let _ = sender.send(AppEvent::RefreshView);
                }
                Err(e) => {
                    let _ = sender.send(AppEvent::Error(format!("Import failed: {}", e)));
                }
            }
            
            let _ = sender.send(AppEvent::LoadingEnd);
        });
    }
}