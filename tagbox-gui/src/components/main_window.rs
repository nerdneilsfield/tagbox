use fltk::{
    prelude::*,
    window::Window,
    group::{Flex, FlexType},
    enums::Color,
};
use std::sync::mpsc::{Receiver, Sender, channel};
use tagbox_core::{config::AppConfig, types::FileEntry};
use crate::state::{AppEvent, AppState};
use crate::components::{
    SearchBar, CategoryTree, FilePreview, FileList, 
    AppMenuBar, StatusBar, DragDropArea, EditDialog
};

pub struct MainWindow {
    window: Window,
    
    // 菜单栏和状态栏
    menu_bar: AppMenuBar,
    pub status_bar: StatusBar,
    
    // 主要组件
    search_bar: SearchBar,
    category_tree: CategoryTree,
    file_list: FileList,
    pub file_preview: FilePreview,
    drag_drop_area: DragDropArea,
    pub edit_dialog: EditDialog,
    
    // 布局容器
    main_container: Flex,
    
    // 状态和事件
    state: AppState,
    pub event_sender: Sender<AppEvent>,
}

impl MainWindow {
    pub fn new(config: AppConfig) -> Result<(Self, Receiver<AppEvent>), Box<dyn std::error::Error>> {
        let (event_sender, event_receiver) = channel();
        
        // 创建主窗口 (1200x850，增加高度容纳菜单栏和状态栏)
        let mut window = Window::new(100, 100, 1200, 850, "TagBox - File Management System");
        window.set_color(Color::from_rgb(245, 245, 245));
        
        // 菜单栏 (顶部 25px)
        let menu_bar = AppMenuBar::new(0, 0, 1200, 25, event_sender.clone());
        
        // 搜索栏 (菜单栏下方 50px)
        let mut search_bar = SearchBar::new(5, 30, 1190, 50, event_sender.clone());
        search_bar.enable_live_suggestions(config.clone());
        
        // 主体布局容器 (搜索栏下方到状态栏上方)
        let mut main_container = Flex::new(5, 85, 1190, 740, None);
        main_container.set_type(FlexType::Row);
        main_container.set_spacing(8);
        
        // 左侧分类树 (25% 宽度)
        let mut category_tree = CategoryTree::new(0, 0, 295, 740, event_sender.clone());
        main_container.fixed(category_tree.widget(), 295);
        
        // 中间区域：文件列表和拖拽区域 (40% 宽度)
        let mut middle_flex = Flex::new(0, 0, 475, 740, None);
        middle_flex.set_type(FlexType::Column);
        middle_flex.set_spacing(8);
        
        // 文件列表 (上方 80%)
        let mut file_list = FileList::new(0, 0, 475, 590, event_sender.clone());
        middle_flex.fixed(file_list.widget(), 590);
        
        // 拖拽区域 (下方 20%)
        let mut drag_drop_area = DragDropArea::new(0, 0, 475, 140, event_sender.clone());
        middle_flex.fixed(drag_drop_area.widget(), 140);
        
        middle_flex.end();
        main_container.fixed(&middle_flex, 475);
        
        // 右侧预览面板 (35% 宽度)
        let mut file_preview = FilePreview::new(0, 0, 415, 740, event_sender.clone());
        main_container.fixed(file_preview.widget(), 415);
        
        main_container.end();
        
        // 状态栏 (底部 25px)
        let status_bar = StatusBar::new(0, 825, 1200, 25, event_sender.clone());
        
        window.end();
        
        // 创建应用状态
        let state = AppState::new(config);
        
        // 设置增强的拖拽支持
        Self::setup_drag_drop(&mut window, event_sender.clone());
        
        // 启用拖拽区域的活动状态
        drag_drop_area.set_active(true);
        
        // 创建编辑对话框
        let mut edit_dialog = EditDialog::new(event_sender.clone());
        edit_dialog.set_callbacks();
        
        Ok((Self {
            window,
            menu_bar,
            status_bar,
            search_bar,
            category_tree,
            file_list,
            file_preview,
            drag_drop_area,
            edit_dialog,
            main_container,
            state,
            event_sender,
        }, event_receiver))
    }
    
    pub fn show(&mut self) {
        self.window.show();
    }
    
    pub fn select_file(&mut self, file_id: String) {
        // 检查是否是索引格式的文件选择
        if file_id.starts_with("index:") {
            if let Ok(index) = file_id.strip_prefix("index:").unwrap().parse::<usize>() {
                // 从文件列表中获取指定索引的文件
                if let Some(file) = self.state.current_files.get(index) {
                    // 更新文件预览
                    self.file_preview.display_file_sync(file);
                    
                    // 更新状态
                    self.state.selected_file = Some(file.clone());
                    
                    // 更新文件列表选择状态
                    self.file_list.select_file_by_index(index);
                    
                    // 更新状态栏
                    self.status_bar.set_file_count(self.state.current_files.len(), Some(1));
                    
                    println!("File selected: {} by index {}", file.title, index);
                }
            }
        } else {
            // 传统的文件ID选择
            self.state.select_file(&file_id);
            // 通过异步桥接更新预览面板
            // 在实际应用中会通过事件系统处理
        }
    }
    
    pub fn update_file_list(&mut self, files: Vec<tagbox_core::types::FileEntry>) {
        use tagbox_core::types::SearchResult;
        
        let search_result = SearchResult {
            entries: files,
            total_count: 0,
            offset: 0,
            limit: 0,
        };
        
        // 异步加载文件到列表中
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Err(e) = self.file_list.load_files(search_result.clone()).await {
                eprintln!("Failed to load files: {}", e);
            }
        });
        
        self.state.set_files(search_result.entries.clone());
        
        // 更新分类树的文件计数
        self.category_tree.update_file_counts(&search_result.entries);
        
        // 更新状态栏
        self.status_bar.set_file_count(self.state.get_files().len(), None);
        self.status_bar.set_temp_status(&format!("✅ Loaded {} files", search_result.entries.len()), 2000);
    }
    
    pub fn set_loading(&mut self, loading: bool) {
        self.state.set_loading(loading);
        self.search_bar.set_loading(loading);
        self.file_preview.set_loading(loading);
        self.file_list.set_loading(loading);
        self.status_bar.set_loading(loading, None);
        
        // 更新状态栏消息
        if loading {
            self.status_bar.set_default_message("Loading...");
        } else {
            self.status_bar.set_default_message("Ready");
        }
    }
    
    // 异步加载分类树
    pub async fn load_categories(&mut self, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        self.category_tree.load_categories(config).await
    }
    
    // 异步显示文件详情
    pub async fn display_file_details_async(&mut self, file_id: &str, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        self.file_preview.display_file(file_id, config).await
    }
    
    // 处理分类选择
    pub fn handle_category_select(&mut self, category_path: String) {
        self.category_tree.select_category(Some(category_path.clone()));
        
        // 根据分类过滤文件列表
        if let Some(filter) = self.category_tree.get_category_filter() {
            let _ = self.event_sender.send(AppEvent::SearchQuery(filter));
        }
    }
    
    // 处理分类展开/折叠
    pub fn handle_category_expand(&mut self, category_path: String) {
        self.category_tree.expand_category(category_path);
    }
    
    // 清除选择
    pub fn clear_selection(&mut self) {
        self.file_list.clear();
        self.file_preview.clear();
        self.category_tree.select_category(None);
    }
    
    // 刷新所有组件
    pub async fn refresh_all(&mut self, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        self.load_categories(config).await?;
        // 重新执行当前搜索
        // TODO: 保存并重新执行当前查询
        Ok(())
    }
    
    // 设置增强的拖拽支持
    fn setup_drag_drop(window: &mut Window, event_sender: Sender<AppEvent>) {
        let sender_clone = event_sender.clone();
        let mut drag_active = false;
        let mut drag_start_time = std::time::Instant::now();
        
        window.handle(move |window, event| {
            match event {
                fltk::enums::Event::DndEnter => {
                    drag_active = true;
                    drag_start_time = std::time::Instant::now();
                    // 更柔和的全窗口拖拽反馈
                    window.set_color(Color::from_rgb(240, 252, 240)); // 非常浅的绿色
                    window.redraw();
                    true
                },
                fltk::enums::Event::DndDrag => {
                    // 动态拖拽反馈 - 根据拖拽时间改变颜色深度
                    if drag_active {
                        let elapsed = drag_start_time.elapsed().as_millis();
                        let intensity = ((elapsed / 100) % 20) as u8; // 0-19 循环
                        let green_value = 240 + intensity / 2; // 240-249 范围
                        window.set_color(Color::from_rgb(240, green_value, 240));
                        window.redraw();
                    }
                    true
                },
                fltk::enums::Event::DndLeave => {
                    if drag_active {
                        drag_active = false;
                        window.set_color(Color::from_rgb(245, 245, 245)); // 恢复原始颜色
                        window.redraw();
                    }
                    false
                },
                fltk::enums::Event::DndRelease => {
                    if drag_active {
                        drag_active = false;
                        
                        // 显示短暂的处理反馈
                        window.set_color(Color::from_rgb(255, 248, 220)); // 浅黄色表示正在处理
                        window.redraw();
                        
                        // 处理多文件拖拽并统计结果
                        let file_paths = Self::parse_drag_data();
                        let total_files = file_paths.len();
                        let mut supported_files = 0;
                        
                        for path in file_paths {
                            if Self::is_supported_file(&path) {
                                let _ = sender_clone.send(AppEvent::FileImport(path));
                                supported_files += 1;
                            } else {
                                println!("Unsupported file type: {}", path.display());
                            }
                        }
                        
                        // 根据结果提供视觉反馈
                        let final_color = if supported_files > 0 {
                            Color::from_rgb(240, 255, 240) // 成功 - 浅绿色
                        } else if total_files > 0 {
                            Color::from_rgb(255, 240, 240) // 失败 - 浅红色
                        } else {
                            Color::from_rgb(245, 245, 245) // 无文件 - 默认色
                        };
                        
                        window.set_color(final_color);
                        window.redraw();
                        
                        // 1.5秒后恢复默认颜色
                        let window_color_reset = std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_millis(1500));
                            // 注意：实际应用中通过事件系统处理颜色重置
                        });
                    }
                    true
                }
                _ => false,
            }
        });
    }
    
    // 解析拖拽数据以支持多文件
    fn parse_drag_data() -> Vec<std::path::PathBuf> {
        let mut paths = Vec::new();
        
        let text = fltk::app::event_text();
        if !text.is_empty() {
            // 处理多行文件路径（Unix/Linux 格式）
            for line in text.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    // 处理 file:// 协议的URI
                    let path_str = if trimmed.starts_with("file://") {
                        &trimmed[7..] // 移除 "file://" 前缀
                    } else {
                        trimmed
                    };
                    
                    let path = std::path::PathBuf::from(path_str);
                    if path.exists() {
                        paths.push(path);
                    }
                }
            }
        }
        
        paths
    }
    
    // 检查文件是否为支持的类型（扩展版）
    fn is_supported_file(path: &std::path::Path) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                match ext_str.to_lowercase().as_str() {
                    "pdf" | "epub" | "txt" | "md" | "doc" | "docx" | 
                    "rtf" | "html" | "htm" | "odt" | "mobi" | "azw" | "azw3" => true,
                    _ => false,
                }
            } else {
                false
            }
        } else {
            // 检查无扩展名的文本文件
            if let Ok(metadata) = std::fs::metadata(path) {
                metadata.is_file() && metadata.len() < 50 * 1024 * 1024 // 小于50MB可能是文本文件
            } else {
                false
            }
        }
    }
    
    // 打开设置对话框
    pub fn open_settings_dialog(&mut self) {
        use crate::components::SettingsDialog;
        use std::path::Path;
        
        let mut dialog = SettingsDialog::new(self.event_sender.clone());
        // 传递配置文件路径
        let config_path = Some(Path::new("config.toml").to_path_buf());
        dialog.load_config(self.state.config.clone(), config_path);
        dialog.show();
        
        // 等待对话框关闭
        while dialog.shown() {
            fltk::app::wait();
        }
    }
    
    // 打开日志查看器对话框
    pub fn open_log_viewer_dialog(&mut self) {
        use crate::components::LogViewer;
        
        let mut log_viewer = LogViewer::new(self.event_sender.clone());
        log_viewer.show();
        
        // 注意：不要在这里等待对话框关闭，因为这会阻塞主UI线程
        // 日志查看器应该作为独立窗口运行
    }
    
    // 显示统计信息对话框
    pub fn show_statistics_dialog(&mut self) {
        // 简单的统计信息显示
        let stats_text = format!(
            "TagBox Statistics\n\n\
            Current Files: {}\n\
            Selected File: {}\n\
            Current Query: \"{}\"\n\
            Database Path: {}\n\
            Storage Path: {}",
            self.state.current_files.len(),
            self.state.selected_file.as_ref()
                .map(|f| f.title.as_str())
                .unwrap_or("None"),
            self.state.current_query,
            self.state.config.database.path.display(),
            self.state.config.import.paths.storage_dir.display()
        );
        
        fltk::dialog::message_default(&stats_text);
    }

    // 显示文件详情
    pub fn display_file_details(&mut self, file: &FileEntry) {
        // 更新文件预览面板
        self.file_preview.display_file_sync(file);
        
        // 更新状态
        self.state.selected_file = Some(file.clone());
        self.state.set_loading(false);
    }

    // 显示导入成功通知（增强版）
    pub fn show_import_success(&mut self, file: &FileEntry) {
        // 更新拖拽区域显示成功状态
        self.drag_drop_area.show_success(&format!("Successfully imported: {}", file.title));
        
        // 同时更新状态栏
        self.status_bar.set_message(&format!("✅ Imported: {}", file.title), false);
        
        // 可选：显示系统通知（不阻塞UI）
        println!("Successfully imported: {} (ID: {})", file.title, file.id);
    }
    
    // 显示导入错误通知
    pub fn show_import_error(&mut self, path: &std::path::Path, error: &str) {
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown file");
            
        // 更新拖拽区域显示错误状态
        self.drag_drop_area.show_error(&format!("Failed to import: {}", filename));
        
        // 同时更新状态栏
        self.status_bar.set_message(&format!("❌ Import failed: {} ({})", filename, error), true);
    }
    
    // 打开编辑对话框
    pub async fn open_edit_dialog(&mut self, file_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 加载文件信息到编辑对话框
        self.edit_dialog.load_file(file_id, &self.state.config).await?;
        
        // 显示对话框
        self.edit_dialog.show();
        
        Ok(())
    }
    
    // 关闭编辑对话框
    pub fn close_edit_dialog(&mut self) {
        self.edit_dialog.hide();
        self.edit_dialog.clear_form();
    }
    
    // 保存文件编辑
    pub async fn save_file_edit(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 验证表单
        self.edit_dialog.validate_form()?;
        
        // 保存更改
        self.edit_dialog.save_changes(&self.state.config).await?;
        
        // 关闭对话框
        self.close_edit_dialog();
        
        // 更新状态栏
        self.status_bar.set_temp_status("✅ File saved successfully", 2000);
        
        // 刷新文件列表
        let _ = self.event_sender.send(AppEvent::RefreshView);
        
        Ok(())
    }
    
    // 删除文件
    pub async fn delete_file_edit(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 执行软删除
        self.edit_dialog.delete_file(&self.state.config).await?;
        
        // 关闭对话框
        self.close_edit_dialog();
        
        // 更新状态栏
        self.status_bar.set_temp_status("🗑️ File deleted", 2000);
        
        // 刷新文件列表
        let _ = self.event_sender.send(AppEvent::RefreshView);
        
        Ok(())
    }
    
    // 批量导入进度更新
    pub fn update_import_progress(&mut self, current: usize, total: usize, current_file: Option<&str>) {
        self.drag_drop_area.show_import_progress(current, total, current_file);
        
        let progress_text = if let Some(filename) = current_file {
            format!("Importing {} ({}/{})", filename, current, total)
        } else {
            format!("Importing files... ({}/{})", current, total)
        };
        
        self.status_bar.set_message(&progress_text, false);
    }
    
    // 完成批量导入
    pub fn complete_batch_import(&mut self, total: usize, successful: usize, failed: usize) {
        self.drag_drop_area.show_import_stats(total, successful, failed);
        
        let status_message = if failed == 0 {
            format!("✅ Successfully imported {} files", successful)
        } else {
            format!("⚠️ Imported {}/{} files ({} failed)", successful, total, failed)
        };
        
        self.status_bar.set_message(&status_message, failed > 0);
    }
    
    // 获取当前文件列表（用于事件处理）
    pub fn get_current_files(&self) -> &Vec<tagbox_core::types::FileEntry> {
        &self.state.current_files
    }
    
    // 更新状态栏（定期调用）
    pub fn update_status_bar(&mut self) {
        self.status_bar.update();
    }
}