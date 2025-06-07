use std::sync::mpsc::Receiver;
use tagbox_core::config::AppConfig;
use crate::state::AppEvent;
use crate::components::MainWindow;
use crate::utils::AsyncBridge;

pub struct App {
    pub main_window: MainWindow,
    pub event_receiver: Receiver<AppEvent>,
    pub async_bridge: AsyncBridge,
    pub config: AppConfig,
}

impl App {
    pub fn new(config: AppConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let (main_window, event_receiver) = MainWindow::new(config.clone())?;
        
        // 从 main_window 获取 event_sender 来创建 async_bridge
        let event_sender = main_window.event_sender.clone();
        let async_bridge = AsyncBridge::with_sender(event_sender);
        
        Ok(Self {
            main_window,
            event_receiver,
            async_bridge,
            config,
        })
    }
    
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let app = fltk::app::App::default();
        self.main_window.show();
        
        // 启动时自动加载所有文件
        tracing::info!("Loading initial file list...");
        self.async_bridge.spawn_load_all_files(self.config.clone());
        
        // 设置定时器来更新状态栏
        let mut last_update = std::time::Instant::now();
        let update_interval = std::time::Duration::from_millis(100); // 100ms更新一次
        
        while app.wait() {
            // 定期更新状态栏
            let now = std::time::Instant::now();
            if now.duration_since(last_update) >= update_interval {
                self.main_window.update_status_bar();
                last_update = now;
            }
            
            // 处理应用事件
            if let Ok(event) = self.event_receiver.try_recv() {
                self.handle_event(event)?;
            }
        }
        
        Ok(())
    }
    
    fn handle_event(&mut self, event: AppEvent) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            AppEvent::SearchQuery(query) => {
                tracing::info!("Performing search: {}", query);
                self.main_window.set_loading(true);
                self.async_bridge.spawn_search(query, self.config.clone());
            }
            AppEvent::SearchResults(result) => {
                tracing::info!("Search completed: {} results", result.entries.len());
                self.main_window.set_loading(false);
                self.main_window.update_file_list(result.entries.clone());
                
                // 更新搜索状态
                if result.entries.is_empty() {
                    self.main_window.status_bar.set_temp_status("⚠️ No results found", 3000);
                } else {
                    self.main_window.status_bar.set_temp_status(&format!("✅ Found {} results", result.entries.len()), 2000);
                }
            }
            AppEvent::FileSelected(file_id) => {
                tracing::info!("File selected: {}", file_id);
                self.main_window.select_file(file_id.clone());
                // 异步加载文件详情
                self.async_bridge.spawn_load_file(file_id, self.config.clone());
            }
            AppEvent::FileImport(path) => {
                tracing::info!("Importing file: {}", path.display());
                self.main_window.set_loading(true);
                
                let filename = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("file");
                self.main_window.status_bar.set_temp_status(&format!("📥 Importing: {}", filename), 1000);
                
                self.async_bridge.spawn_import_file(path, self.config.clone());
            }
            AppEvent::CategorySelect(category_path) => {
                tracing::info!("Category selected: {}", category_path);
                self.main_window.handle_category_select(category_path.clone());
                self.main_window.set_loading(true);
                self.main_window.status_bar.set_temp_status(&format!("📋 Filtering by: {}", category_path), 1000);
                self.async_bridge.spawn_category_search(category_path, self.config.clone());
            }
            AppEvent::CategoryExpand(category_path) => {
                tracing::info!("Category expanded: {}", category_path);
                self.main_window.handle_category_expand(category_path);
            }
            AppEvent::LoadingStart => {
                self.main_window.set_loading(true);
            }
            AppEvent::LoadingEnd => {
                self.main_window.set_loading(false);
            }
            AppEvent::RefreshView => {
                tracing::info!("Refreshing view");
                // 异步刷新所有组件
                let config = self.config.clone();
                let sender = self.main_window.event_sender.clone();
                self.async_bridge.runtime.spawn(async move {
                    // 发送刷新完成事件
                    let _ = sender.send(AppEvent::LoadingEnd);
                });
            }
            AppEvent::FileOpen(file_id) => {
                tracing::info!("Opening file: {}", file_id);
                // 打开当前选中的文件
                if let Some(file) = self.main_window.file_preview.get_current_file() {
                    if let Err(e) = crate::utils::open_file(&file.path) {
                        let _ = self.main_window.event_sender.send(AppEvent::Error(format!("Failed to open file: {}", e)));
                    }
                }
            }
            AppEvent::Error(msg) => {
                tracing::error!("Application error: {}", msg);
                self.main_window.set_loading(false);
                self.main_window.status_bar.set_message(&format!("❌ Error: {}", msg), true);
                fltk::dialog::alert_default(&format!("Error: {}", msg));
            }
            AppEvent::OpenSettings => {
                tracing::info!("Opening settings dialog");
                self.main_window.open_settings_dialog();
            }
            AppEvent::OpenLogViewer => {
                tracing::info!("Opening log viewer");
                self.main_window.open_log_viewer_dialog();
            }
            AppEvent::ShowStatistics => {
                tracing::info!("Showing statistics");
                self.main_window.show_statistics_dialog();
            }
            AppEvent::FileLoaded(file) => {
                tracing::info!("File loaded: {}", file.title);
                self.main_window.display_file_details(&file);
            }
            AppEvent::FileImported(file) => {
                tracing::info!("File imported: {}", file.title);
                // 显示导入成功的通知
                self.main_window.show_import_success(&file);
                self.main_window.status_bar.set_temp_status(&format!("✅ Imported: {}", file.title), 3000);
            }
            AppEvent::AdvancedSearch(options) => {
                tracing::info!("Performing advanced search");
                self.main_window.set_loading(true);
                self.async_bridge.spawn_advanced_search(options, self.config.clone());
            }
            // 右键菜单事件处理
            AppEvent::OpenFile(file_ref) => {
                tracing::info!("Opening file: {}", file_ref);
                if let Some(file) = self.get_file_by_ref(&file_ref) {
                    let file_title = file.title.clone();
                    let file_path = file.path.clone();
                    
                    self.main_window.status_bar.set_temp_status(&format!("📄 Opening: {}", file_title), 1500);
                    if let Err(e) = crate::utils::open_file(&file_path) {
                        let _ = self.main_window.event_sender.send(AppEvent::Error(format!("Failed to open file: {}", e)));
                    }
                }
            }
            AppEvent::EditFile(file_ref) => {
                tracing::info!("Editing file metadata: {}", file_ref);
                if let Some(file) = self.get_file_by_ref(&file_ref) {
                    // TODO: 打开文件编辑对话框
                    println!("Opening edit dialog for: {}", file.title);
                }
            }
            AppEvent::CopyFilePath(file_ref) => {
                tracing::info!("Copying file path: {}", file_ref);
                if let Some(file) = self.get_file_by_ref(&file_ref) {
                    if let Err(e) = crate::utils::copy_to_clipboard(&file.path.to_string_lossy()) {
                        let _ = self.main_window.event_sender.send(AppEvent::Error(format!("Failed to copy path: {}", e)));
                    } else {
                        self.main_window.status_bar.set_temp_status("📋 Path copied to clipboard", 2000);
                    }
                }
            }
            AppEvent::ShowInFolder(file_ref) => {
                tracing::info!("Showing file in folder: {}", file_ref);
                if let Some(file) = self.get_file_by_ref(&file_ref) {
                    let file_title = file.title.clone();
                    let file_path = file.path.clone();
                    
                    self.main_window.status_bar.set_temp_status(&format!("📁 Opening folder: {}", file_title), 1500);
                    if let Err(e) = crate::utils::open_folder(&file_path) {
                        let _ = self.main_window.event_sender.send(AppEvent::Error(format!("Failed to open folder: {}", e)));
                    }
                }
            }
            AppEvent::DeleteFile(file_ref) => {
                tracing::info!("Deleting file: {}", file_ref);
                if let Some(file) = self.get_file_by_ref(&file_ref) {
                    // TODO: 实现文件删除功能
                    println!("Deleting file: {} (ID: {})", file.title, file.id);
                }
            }
            _ => {
                tracing::debug!("Unhandled event: {:?}", event);
            }
        }
        Ok(())
    }
    
    // 辅助方法：根据文件引用获取文件
    fn get_file_by_ref(&self, file_ref: &str) -> Option<&tagbox_core::types::FileEntry> {
        if file_ref.starts_with("index:") {
            if let Ok(index) = file_ref.strip_prefix("index:").unwrap().parse::<usize>() {
                self.main_window.get_current_files().get(index)
            } else {
                None
            }
        } else {
            // 按ID查找
            self.main_window.get_current_files().iter().find(|f| f.id == file_ref)
        }
    }
}