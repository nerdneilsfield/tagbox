use fltk::{
    prelude::*,
    window::Window,
    group::{Flex, FlexType},
    enums::Color,
};
use std::sync::mpsc::{Receiver, Sender, channel};
use tagbox_core::config::AppConfig;
use crate::state::{AppEvent, AppState};
use crate::components::{
    SearchBar, CategoryTree, FilePreview, FileList, 
    AppMenuBar, StatusBar, DragDropArea
};

pub struct MainWindow {
    window: Window,
    
    // 菜单栏和状态栏
    menu_bar: AppMenuBar,
    status_bar: StatusBar,
    
    // 主要组件
    search_bar: SearchBar,
    category_tree: CategoryTree,
    file_list: FileList,
    pub file_preview: FilePreview,
    drag_drop_area: DragDropArea,
    
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
        window.set_color(Color::from_rgb(248, 249, 250));
        
        // 菜单栏 (顶部 25px)
        let menu_bar = AppMenuBar::new(0, 0, 1200, 25, event_sender.clone());
        
        // 搜索栏 (菜单栏下方 50px)
        let search_bar = SearchBar::new(10, 30, 1180, 50, event_sender.clone());
        
        // 主体布局容器 (搜索栏下方到状态栏上方)
        let mut main_container = Flex::new(10, 85, 1180, 740, None);
        main_container.set_type(FlexType::Row);
        main_container.set_spacing(5);
        
        // 左侧分类树 (25% 宽度)
        let mut category_tree = CategoryTree::new(0, 0, 295, 740, event_sender.clone());
        main_container.fixed(category_tree.widget(), 295);
        
        // 中间区域：文件列表和拖拽区域 (40% 宽度)
        let mut middle_flex = Flex::new(0, 0, 470, 740, None);
        middle_flex.set_type(FlexType::Column);
        middle_flex.set_spacing(5);
        
        // 文件列表 (上方 80%)
        let mut file_list = FileList::new(0, 0, 470, 590, event_sender.clone());
        middle_flex.fixed(file_list.widget(), 590);
        
        // 拖拽区域 (下方 20%)
        let mut drag_drop_area = DragDropArea::new(0, 0, 470, 145, event_sender.clone());
        middle_flex.fixed(drag_drop_area.widget(), 145);
        
        middle_flex.end();
        main_container.fixed(&middle_flex, 470);
        
        // 右侧预览面板 (35% 宽度)
        let mut file_preview = FilePreview::new(0, 0, 410, 740, event_sender.clone());
        main_container.fixed(file_preview.widget(), 410);
        
        main_container.end();
        
        // 状态栏 (底部 25px)
        let status_bar = StatusBar::new(0, 825, 1200, 25, event_sender.clone());
        
        window.end();
        
        // 创建应用状态
        let state = AppState::new(config);
        
        // 设置增强的拖拽支持
        Self::setup_drag_drop(&mut window, event_sender.clone());
        
        Ok((Self {
            window,
            menu_bar,
            status_bar,
            search_bar,
            category_tree,
            file_list,
            file_preview,
            drag_drop_area,
            main_container,
            state,
            event_sender,
        }, event_receiver))
    }
    
    pub fn show(&mut self) {
        self.window.show();
    }
    
    pub fn select_file(&mut self, file_id: String) {
        self.state.select_file(&file_id);
        // 通过异步桥接更新预览面板
        // 在实际应用中会通过事件系统处理
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
        
        self.state.set_files(search_result.entries);
        
        // 更新状态栏
        self.status_bar.set_file_count(self.state.get_files().len(), None);
    }
    
    pub fn set_loading(&mut self, loading: bool) {
        self.state.set_loading(loading);
        self.search_bar.set_loading(loading);
        self.file_preview.set_loading(loading);
        self.file_list.set_loading(loading);
        self.status_bar.set_loading(loading, None);
    }
    
    // 异步加载分类树
    pub async fn load_categories(&mut self, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        self.category_tree.load_categories(config).await
    }
    
    // 显示文件详情
    pub async fn display_file_details(&mut self, file_id: &str, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
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
    
    // 设置拖拽支持
    fn setup_drag_drop(window: &mut Window, event_sender: Sender<AppEvent>) {
        let sender_clone = event_sender.clone();
        let mut drag_active = false;
        
        window.handle(move |window, event| {
            match event {
                fltk::enums::Event::DndEnter => {
                    drag_active = true;
                    window.set_color(Color::from_rgb(220, 248, 220)); // 浅绿色拖拽反馈
                    window.redraw();
                    true
                },
                fltk::enums::Event::DndDrag => {
                    true
                },
                fltk::enums::Event::DndLeave => {
                    if drag_active {
                        drag_active = false;
                        window.set_color(Color::from_rgb(248, 249, 250)); // 恢复原始颜色
                        window.redraw();
                    }
                    false
                },
                fltk::enums::Event::DndRelease => {
                    if drag_active {
                        drag_active = false;
                        window.set_color(Color::from_rgb(248, 249, 250)); // 恢复原始颜色
                        window.redraw();
                        
                        // 处理多文件拖拽
                        let file_paths = Self::parse_drag_data();
                        for path in file_paths {
                            if Self::is_supported_file(&path) {
                                let _ = sender_clone.send(AppEvent::FileImport(path));
                            } else {
                                println!("Unsupported file type: {}", path.display());
                            }
                        }
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
    
    // 检查文件是否为支持的类型
    fn is_supported_file(path: &std::path::Path) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                match ext_str.to_lowercase().as_str() {
                    "pdf" | "epub" | "txt" | "md" | "doc" | "docx" => true,
                    _ => false,
                }
            } else {
                false
            }
        } else {
            false
        }
    }
    
    // 打开设置对话框
    pub fn open_settings_dialog(&mut self) {
        use crate::components::SettingsDialog;
        
        let mut dialog = SettingsDialog::new(self.event_sender.clone());
        dialog.load_config(self.state.config.clone(), None);
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
        
        // 等待对话框关闭
        while log_viewer.shown() {
            fltk::app::wait();
        }
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
}