use fltk::{
    prelude::*,
    browser::Browser,
    enums::{Color, Event},
    group::Group,
    menu::{MenuButton, MenuFlag},
    app::MouseButton,
};
use std::sync::mpsc::Sender;
use tagbox_core::types::{FileEntry, SearchResult};
use crate::state::AppEvent;

pub struct FileList {
    container: Group,
    browser: Browser,
    files: Vec<FileEntry>,
    selected_index: Option<usize>,
    event_sender: Sender<AppEvent>,
}

impl FileList {
    pub fn new(
        x: i32, 
        y: i32, 
        w: i32, 
        h: i32, 
        event_sender: Sender<AppEvent>
    ) -> Self {
        let mut container = Group::new(x, y, w, h, None);
        
        let mut browser = Browser::new(x, y, w, h, None);
        browser.set_color(Color::White);
        browser.set_selection_color(Color::from_rgb(230, 240, 255));
        
        container.end();
        
        let mut file_list = Self {
            container,
            browser,
            files: Vec::new(),
            selected_index: None,
            event_sender,
        };
        
        file_list.setup_callbacks();
        file_list.setup_drag_drop();
        file_list
    }
    
    fn setup_callbacks(&mut self) {
        let sender = self.event_sender.clone();
        let sender_menu = self.event_sender.clone();
        
        // 正常选择回调
        self.browser.set_callback(move |browser| {
            let selected = browser.value();
            if selected > 0 {
                // 为了简化起见，我们发送选中的索引，然后在主窗口中处理
                let _ = sender.send(AppEvent::FileSelected(format!("index:{}", selected - 1)));
            }
        });
        
        // 右键菜单处理
        self.browser.handle(move |browser, event| {
            match event {
                Event::Push => {
                    if fltk::app::event_mouse_button() == MouseButton::Right {
                        let selected = browser.value();
                        if selected > 0 {
                            // 显示右键菜单
                            Self::show_context_menu((selected - 1) as usize, &sender_menu);
                        }
                        true
                    } else {
                        false
                    }
                },
                _ => false,
            }
        });
    }
    
    pub async fn load_files(&mut self, search_result: SearchResult) -> Result<(), Box<dyn std::error::Error>> {
        self.files = search_result.entries;
        
        // 清空浏览器
        self.browser.clear();
        
        if self.files.is_empty() {
            // 显示空状态提示
            self.browser.add("No files found. Try a different search or import some files.");
            self.browser.deactivate();
            return Ok(());
        }
        
        // 激活浏览器
        self.browser.activate();
        
        // 添加文件到浏览器，使用改进的格式
        for (index, file) in self.files.iter().enumerate() {
            let display_title = if file.title.is_empty() {
                &file.original_filename
            } else {
                &file.title
            };
            
            // 限制标题长度以保持格式整洁（按字符数而不是字节数）
            let title = if display_title.chars().count() > 40 {
                let truncated: String = display_title.chars().take(37).collect();
                format!("{}...", truncated)
            } else {
                display_title.to_string()
            };
            
            let authors_str = if file.authors.is_empty() {
                "Unknown".to_string()
            } else {
                let authors = file.authors.join(", ");
                if authors.chars().count() > 25 {
                    let truncated: String = authors.chars().take(22).collect();
                    format!("{}...", truncated)
                } else {
                    authors
                }
            };
            
            let year_str = file.year.map(|y| y.to_string()).unwrap_or_else(|| "----".to_string());
            
            let tags_count = file.tags.len();
            let tags_str = if tags_count == 0 {
                "No tags".to_string()
            } else if tags_count == 1 {
                file.tags[0].clone()
            } else {
                format!("{} tags", tags_count)
            };
            
            // 使用固定宽度格式，增加可读性
            let line = format!("{:3}: {:40} | {:25} | {:4} | {}", 
                index + 1, title, authors_str, year_str, tags_str);
            
            self.browser.add(&line);
        }
        
        println!("Loaded {} files into file list", self.files.len());
        Ok(())
    }
    
    pub fn clear(&mut self) {
        self.files.clear();
        self.selected_index = None;
        self.browser.clear();
    }
    
    pub fn get_selected_file(&self) -> Option<&FileEntry> {
        if let Some(index) = self.selected_index {
            self.files.get(index)
        } else {
            None
        }
    }
    
    pub fn select_file(&mut self, file_id: &str) {
        for (index, file) in self.files.iter().enumerate() {
            if file.id == file_id {
                self.selected_index = Some(index);
                self.browser.select(index as i32 + 1);
                println!("Selected file: {} (index: {})", file.title, index);
                break;
            }
        }
    }
    
    // 根据索引选择文件
    pub fn select_file_by_index(&mut self, index: usize) {
        if index < self.files.len() {
            self.selected_index = Some(index);
            self.browser.select(index as i32 + 1);
            if let Some(file) = self.files.get(index) {
                println!("Selected file by index: {} (index: {})", file.title, index);
            }
        }
    }
    
    // 获取当前选中的文件
    pub fn get_current_selection(&self) -> Option<&FileEntry> {
        if let Some(index) = self.selected_index {
            self.files.get(index)
        } else {
            None
        }
    }
    
    pub fn refresh(&mut self) {
        self.browser.redraw();
    }
    
    pub fn set_loading(&mut self, loading: bool) {
        if loading {
            self.browser.deactivate();
        } else {
            self.browser.activate();
        }
    }
    
    pub fn get_file_stats(&self) -> usize {
        self.files.len()
    }
    
    // 获取容器引用（用于主窗口布局）
    pub fn widget(&mut self) -> &mut Group {
        &mut self.container
    }
    
    // 设置拖拽支持
    fn setup_drag_drop(&mut self) {
        let sender = self.event_sender.clone();
        let mut drag_overlay_shown = false;
        
        self.browser.handle(move |browser, event| {
            match event {
                fltk::enums::Event::DndEnter => {
                    drag_overlay_shown = true;
                    browser.set_color(Color::from_rgb(240, 248, 255)); // 浅蓝色拖拽反馈
                    browser.redraw();
                    true
                },
                fltk::enums::Event::DndDrag => {
                    true
                },
                fltk::enums::Event::DndLeave => {
                    if drag_overlay_shown {
                        drag_overlay_shown = false;
                        browser.set_color(Color::White); // 恢复原始颜色
                        browser.redraw();
                    }
                    false
                },
                fltk::enums::Event::DndRelease => {
                    if drag_overlay_shown {
                        drag_overlay_shown = false;
                        browser.set_color(Color::White); // 恢复原始颜色
                        browser.redraw();
                        
                        // 处理拖拽导入
                        Self::handle_file_drop(&sender);
                    }
                    true
                }
                _ => false,
            }
        });
    }
    
    // 处理文件拖拽
    fn handle_file_drop(sender: &Sender<AppEvent>) {
        let text = fltk::app::event_text();
        if !text.is_empty() {
            for line in text.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    let path_str = if trimmed.starts_with("file://") {
                        &trimmed[7..]
                    } else {
                        trimmed
                    };
                    
                    let path = std::path::PathBuf::from(path_str);
                    if path.exists() && Self::is_supported_file_type(&path) {
                        let _ = sender.send(AppEvent::FileImport(path));
                    }
                }
            }
        }
    }
    
    // 检查文件类型
    fn is_supported_file_type(path: &std::path::Path) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                matches!(ext_str.to_lowercase().as_str(), 
                    "pdf" | "epub" | "txt" | "md" | "doc" | "docx" | "rtf" | "html" | "htm"
                )
            } else {
                false
            }
        } else {
            false
        }
    }
    
    // 显示拖拽提示
    pub fn show_drag_hint(&mut self, show: bool) {
        if show {
            self.browser.add("Drag files here to import...");
            self.browser.set_color(Color::from_rgb(248, 248, 248));
        } else {
            self.browser.clear();
            self.browser.set_color(Color::White);
        }
        self.browser.redraw();
    }
    
    // 显示右键上下文菜单
    fn show_context_menu(file_index: usize, sender: &Sender<AppEvent>) {
        let mut menu = MenuButton::default();
        let sender_open = sender.clone();
        let sender_edit = sender.clone();
        let sender_copy = sender.clone();
        let sender_folder = sender.clone();
        let sender_delete = sender.clone();
        
        menu.add_choice("📄 Open File");
        menu.add_choice("✏️ Edit Metadata");
        menu.add_choice("📋 Copy Path");
        menu.add_choice("📁 Show in Folder");
        menu.add_choice("➖ Remove");
        
        // 菜单选项处理
        menu.set_callback(move |m| {
            let choice = m.value();
            match choice {
                0 => { // Open File
                    let _ = sender_open.send(AppEvent::OpenFile(format!("index:{}", file_index)));
                },
                1 => { // Edit Metadata
                    let _ = sender_edit.send(AppEvent::EditFile(format!("index:{}", file_index)));
                },
                2 => { // Copy Path
                    let _ = sender_copy.send(AppEvent::CopyFilePath(format!("index:{}", file_index)));
                },
                3 => { // Show in Folder
                    let _ = sender_folder.send(AppEvent::ShowInFolder(format!("index:{}", file_index)));
                },
                4 => { // Remove
                    if fltk::dialog::choice2_default("Remove this file from TagBox?", "Cancel", "Remove", "") == Some(1) {
                        let _ = sender_delete.send(AppEvent::DeleteFile(format!("index:{}", file_index)));
                    }
                },
                _ => {}
            }
        });
        
        // 显示菜单
        menu.popup();
    }
}