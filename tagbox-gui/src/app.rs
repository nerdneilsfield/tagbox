use std::sync::mpsc::Receiver;
use tagbox_core::config::AppConfig;
use crate::state::AppEvent;
use crate::components::MainWindow;
use crate::utils::AsyncBridge;
use crate::themes::{ThemeManager, AppTheme};

pub struct App {
    pub main_window: MainWindow,
    pub event_receiver: Receiver<AppEvent>,
    pub async_bridge: AsyncBridge,
    pub config: AppConfig,
    pub editing_file_id: Option<String>, // 跟踪当前正在编辑的文件
    pub theme_manager: ThemeManager,
}

impl App {
    pub fn new(config: AppConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // 首先初始化主题管理器并应用默认主题
        let mut theme_manager = ThemeManager::new();
        theme_manager.apply_theme(AppTheme::Light); // 使用浅色主题作为默认
        
        let (main_window, event_receiver) = MainWindow::new(config.clone())?;
        
        // 从 main_window 获取 event_sender 来创建 async_bridge
        let event_sender = main_window.event_sender.clone();
        let async_bridge = AsyncBridge::with_sender(event_sender);
        
        // 应用文件管理器专用样式
        theme_manager.apply_file_manager_styling();
        
        Ok(Self {
            main_window,
            event_receiver,
            async_bridge,
            config,
            editing_file_id: None,
            theme_manager,
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
                
                // 检查是否是为了编辑而加载的文件
                if let Some(editing_id) = &self.editing_file_id {
                    if *editing_id == file.id {
                        // 为编辑而加载的文件，在主线程中打开编辑对话框
                        tracing::info!("Opening edit dialog for file: {} - {}", file.id, file.title);
                        
                        // 填充编辑对话框的表单
                        self.main_window.edit_dialog.populate_form(&file);
                        
                        // 显示编辑对话框
                        self.main_window.edit_dialog.show();
                        
                        // 更新状态栏
                        self.main_window.status_bar.set_temp_status(&format!("✏️ Editing: {}", file.title), 2000);
                        
                        return Ok(());
                    }
                }
                
                // 普通的文件加载，显示详情
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
                    let file_id = file.id.clone();
                    let file_title = file.title.clone();
                    
                    // 在新的异步任务中打开编辑对话框
                    let sender = self.main_window.event_sender.clone();
                    self.async_bridge.runtime.spawn(async move {
                        // 发送事件通知主窗口打开编辑对话框
                        match sender.send(AppEvent::FileEdit(file_id)) {
                            Ok(_) => println!("Opening edit dialog for: {}", file_title),
                            Err(e) => eprintln!("Failed to send edit event: {}", e),
                        }
                    });
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
                
                // 检查是否是从编辑对话框发起的删除
                if file_ref == "editing" {
                    if let Some(editing_id) = &self.editing_file_id {
                        // 从编辑对话框删除
                        self.async_bridge.spawn_delete_file(editing_id.clone(), self.config.clone());
                        
                        // 关闭编辑对话框
                        self.main_window.close_edit_dialog();
                        self.editing_file_id = None;
                        
                        // 更新状态栏
                        self.main_window.status_bar.set_temp_status("🗑️ Deleting file...", 2000);
                        return Ok(());
                    }
                }
                
                // 从其他地方发起的删除（如右键菜单）
                if let Some(file) = self.get_file_by_ref(&file_ref) {
                    self.async_bridge.spawn_delete_file(file.id.clone(), self.config.clone());
                    self.main_window.status_bar.set_temp_status(&format!("🗑️ Deleting: {}", file.title), 2000);
                }
            }
            AppEvent::FileEdit(file_id) => {
                tracing::info!("Opening edit dialog for file: {}", file_id);
                self.editing_file_id = Some(file_id.clone());
                self.async_bridge.spawn_open_edit_dialog(file_id, self.config.clone());
            }
            AppEvent::SaveFile => {
                tracing::info!("Saving file changes");
                
                if let Some(file_id) = &self.editing_file_id {
                    // 从编辑对话框收集表单数据
                    let metadata = self.main_window.edit_dialog.collect_form_data();
                    
                    // 验证表单
                    match self.main_window.edit_dialog.validate_form() {
                        Ok(_) => {
                            self.async_bridge.spawn_save_file_edit(file_id.clone(), metadata, self.config.clone());
                            
                            // 关闭编辑对话框
                            self.main_window.close_edit_dialog();
                            self.editing_file_id = None;
                            
                            // 更新状态栏
                            self.main_window.status_bar.set_temp_status("💾 Saving file changes...", 2000);
                        }
                        Err(e) => {
                            // 显示验证错误
                            self.main_window.status_bar.set_temp_status(&format!("❌ Validation error: {}", e), 3000);
                        }
                    }
                } else {
                    tracing::warn!("SaveFile event received but no file is being edited");
                }
            }
            AppEvent::CancelEdit => {
                tracing::info!("Cancelling file edit");
                
                // 关闭编辑对话框并清理状态
                self.main_window.close_edit_dialog();
                self.editing_file_id = None;
                
                // 更新状态栏
                self.main_window.status_bar.set_temp_status("📄 Edit cancelled", 1500);
            }
            AppEvent::FocusSearchBar => {
                tracing::info!("Focusing search bar");
                // 聚焦搜索栏
                self.main_window.focus_search_bar();
            }
            AppEvent::EditSelectedFile => {
                tracing::info!("Editing selected file");
                // 编辑当前选中的文件
                if let Some(selected_file) = self.main_window.get_selected_file() {
                    self.editing_file_id = Some(selected_file.id.clone());
                    self.async_bridge.spawn_open_edit_dialog(selected_file.id.clone(), self.config.clone());
                } else {
                    self.main_window.status_bar.set_temp_status("⚠️ No file selected", 2000);
                }
            }
            AppEvent::DeleteSelectedFile => {
                tracing::info!("Deleting selected file");
                // 删除当前选中的文件
                if let Some(selected_file) = self.main_window.get_selected_file() {
                    // 显示确认对话框
                    let choice = fltk::dialog::choice2_default(
                        &format!("Are you sure you want to delete '{}'?", selected_file.title),
                        "Yes",
                        "No",
                        ""
                    );
                    
                    if choice == Some(0) {
                        self.async_bridge.spawn_delete_file(selected_file.id.clone(), self.config.clone());
                        self.main_window.status_bar.set_temp_status(&format!("🗑️ Deleting: {}", selected_file.title), 2000);
                    }
                } else {
                    self.main_window.status_bar.set_temp_status("⚠️ No file selected", 2000);
                }
            }
            AppEvent::OpenAdvancedSearch => {
                tracing::info!("Opening advanced search dialog");
                self.main_window.open_advanced_search_dialog();
            }
            AppEvent::OpenCategoryManager => {
                tracing::info!("Opening category manager");
                self.main_window.open_category_manager();
            }
            AppEvent::CategoryCreated(category_name) => {
                tracing::info!("Category created: {}", category_name);
                self.main_window.status_bar.set_temp_status(&format!("✅ Category '{}' created", category_name), 2000);
                // 刷新分类树
                let _ = self.main_window.event_sender.send(AppEvent::RefreshView);
            }
            AppEvent::CategoryUpdated(category_name) => {
                tracing::info!("Category updated: {}", category_name);
                self.main_window.status_bar.set_temp_status(&format!("✏️ Category '{}' updated", category_name), 2000);
                // 刷新分类树
                let _ = self.main_window.event_sender.send(AppEvent::RefreshView);
            }
            AppEvent::CategoryDeleted(category_name) => {
                tracing::info!("Category deleted: {}", category_name);
                self.main_window.status_bar.set_temp_status(&format!("🗑️ Category '{}' deleted", category_name), 2000);
                // 刷新分类树
                let _ = self.main_window.event_sender.send(AppEvent::RefreshView);
            }
            AppEvent::ConfigUpdated(config_path) => {
                tracing::info!("Config updated: {}", config_path.display());
                
                // 重新加载配置
                let rt = tokio::runtime::Runtime::new().unwrap();
                match rt.block_on(async { tagbox_core::load_config(&config_path).await }) {
                    Ok(new_config) => {
                        self.config = new_config;
                        self.main_window.status_bar.set_temp_status(&format!("✅ Config loaded: {}", config_path.file_name().unwrap_or_default().to_string_lossy()), 3000);
                        
                        // 刷新视图以应用新配置
                        let _ = self.main_window.event_sender.send(AppEvent::RefreshView);
                        
                        tracing::info!("Successfully reloaded config from: {}", config_path.display());
                    },
                    Err(e) => {
                        let error_msg = format!("Failed to reload config: {}", e);
                        tracing::error!("{}", error_msg);
                        self.main_window.status_bar.set_temp_status(&format!("❌ {}", error_msg), 5000);
                    }
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