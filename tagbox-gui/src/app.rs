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
        
        while app.wait() {
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
                self.main_window.update_file_list(result.entries);
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
                self.async_bridge.spawn_import_file(path, self.config.clone());
            }
            AppEvent::CategorySelect(category_path) => {
                tracing::info!("Category selected: {}", category_path);
                self.main_window.handle_category_select(category_path);
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
            _ => {
                tracing::debug!("Unhandled event: {:?}", event);
            }
        }
        Ok(())
    }
}