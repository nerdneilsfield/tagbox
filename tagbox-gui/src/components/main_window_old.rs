use fltk::{
    prelude::*,
    window::Window,
    group::{Flex, Group, FlexType},
    browser::HoldBrowser,
    enums::Color,
    dialog::NativeFileChooser,
};
use std::sync::mpsc::{Receiver, Sender, channel};
use tagbox_core::config::AppConfig;
use crate::state::{AppEvent, AppState};
use crate::components::{SearchBar, CategoryTree, FilePreview};

pub struct MainWindow {
    window: Window,
    
    // 组件
    search_bar: SearchBar,
    category_tree: CategoryTree,
    file_list: HoldBrowser,
    file_preview: FilePreview,
    
    // 布局容器
    main_container: Flex,
    
    // 状态和事件
    state: AppState,
    pub event_sender: Sender<AppEvent>,
}

impl MainWindow {
    pub fn new(config: AppConfig) -> Result<(Self, Receiver<AppEvent>), Box<dyn std::error::Error>> {
        let (event_sender, event_receiver) = channel();
        
        // 创建主窗口 (1200x800)
        let mut window = Window::new(100, 100, 1200, 800, "TagBox - File Management System");
        window.set_color(Color::from_rgb(248, 249, 250));
        
        // 顶部搜索栏容器 (高度: 60px)
        let mut search_container = Flex::new(10, 10, 1180, 50, None);
        search_container.set_type(FlexType::Row);
        search_container.set_spacing(10);
        
        // 搜索输入框 (70% 宽度)
        let mut search_input = Input::new(0, 0, 0, 0, None);
        search_input.set_value("Search (e.g. tag:Rust -tag:旧版)");
        search_input.set_text_color(Color::from_rgb(128, 128, 128));
        
        // 高级搜索按钮 (15% 宽度)
        let mut advanced_btn = Button::new(0, 0, 0, 0, "Advanced");
        advanced_btn.set_color(Color::from_rgb(0, 123, 255));
        advanced_btn.set_label_color(Color::White);
        
        // 导入按钮 (15% 宽度)
        let mut import_btn = Button::new(0, 0, 0, 0, "Import File");
        import_btn.set_color(Color::from_rgb(40, 167, 69));
        import_btn.set_label_color(Color::White);
        
        // 设置搜索栏布局比例
        search_container.fixed(&search_input, 70);
        search_container.fixed(&advanced_btn, 15);
        search_container.fixed(&import_btn, 15);
        
        search_container.end();
        
        // 主体三栏布局容器 (剩余空间)
        let mut main_container = Flex::new(10, 70, 1180, 720, None);
        main_container.set_type(FlexType::Row);
        main_container.set_spacing(5);
        
        // 左侧分类树 (25% 宽度)
        let mut category_tree = Tree::new(0, 0, 0, 0, None);
        category_tree.set_show_root(false);
        category_tree.set_color(Color::White);
        
        // 中间文件列表 (40% 宽度)
        let mut file_list = HoldBrowser::new(0, 0, 0, 0, None);
        file_list.set_color(Color::White);
        
        // 右侧预览面板 (35% 宽度)
        let mut preview_panel = Group::new(0, 0, 0, 0, None);
        preview_panel.set_color(Color::from_rgb(248, 249, 250));
        preview_panel.end();
        
        // 设置主布局比例
        main_container.fixed(&category_tree, 25);
        main_container.fixed(&file_list, 40);
        main_container.fixed(&preview_panel, 35);
        
        main_container.end();
        window.end();
        
        // 创建应用状态
        let state = AppState::new(config);
        
        // 设置事件回调
        let sender_clone = event_sender.clone();
        search_input.set_callback(move |input| {
            let key = fltk::app::event_key();
            if key == Key::Enter {
                let query = input.value();
                if !query.trim().is_empty() {
                    let _ = sender_clone.send(AppEvent::SearchQuery(query));
                }
            }
        });
        
        let sender_clone = event_sender.clone();
        import_btn.set_callback(move |_| {
            // 打开文件选择对话框
            let mut dialog = NativeFileChooser::new(fltk::dialog::NativeFileChooserType::BrowseFile);
            dialog.set_title("Select file to import");
            dialog.set_filter("All Files\t*");
            dialog.show();
            
            let path = dialog.filename();
            if !path.to_string_lossy().is_empty() {
                let _ = sender_clone.send(AppEvent::FileImport(path));
            }
        });
        
        let sender_clone = event_sender.clone();
        file_list.set_callback(move |browser| {
            let selected = browser.value();
            if selected > 0 {
                if let Some(text) = browser.text(selected) {
                    // 从文本中提取文件ID（这里需要根据实际格式解析）
                    // 暂时使用文本作为文件ID
                    let _ = sender_clone.send(AppEvent::FileSelected(text));
                }
            }
        });
        
        Ok((Self {
            window,
            search_container,
            search_input,
            advanced_btn,
            import_btn,
            main_container,
            category_tree,
            file_list,
            preview_panel,
            state,
            event_sender,
        }, event_receiver))
    }
    
    pub fn show(&mut self) {
        self.window.show();
    }
    
    pub fn select_file(&mut self, file_id: String) {
        self.state.select_file(&file_id);
        // 更新预览面板显示
        self.update_preview();
    }
    
    pub fn update_file_list(&mut self, files: Vec<tagbox_core::types::FileEntry>) {
        self.file_list.clear();
        for file in &files {
            let display_text = format!("{} - {}", 
                if file.title.is_empty() { &file.original_filename } else { &file.title },
                file.id
            );
            self.file_list.add(&display_text);
        }
        self.state.set_files(files);
    }
    
    pub fn set_loading(&mut self, loading: bool) {
        self.state.set_loading(loading);
        if loading {
            self.search_input.set_value("Searching...");
            self.search_input.deactivate();
        } else {
            self.search_input.set_value("");
            self.search_input.activate();
        }
    }
    
    fn update_preview(&mut self) {
        // 这里后续会实现预览面板的更新逻辑
        // 暂时只是占位符
        if let Some(file) = &self.state.selected_file {
            tracing::info!("Selected file: {} ({})", file.original_filename, file.id);
        }
    }
}