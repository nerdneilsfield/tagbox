use fltk::{
    prelude::*,
    browser::MultiBrowser,
    enums::{Color, Event},
    group::Group,
    menu::MenuButton,
    app::MouseButton,
};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use tagbox_core::types::{FileEntry, SearchResult};
use crate::state::AppEvent;

pub struct FileList {
    container: Group,
    browser: MultiBrowser,
    files: Arc<Mutex<Vec<FileEntry>>>,
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
        let container = Group::new(x, y, w, h, None);
        
        // 使用 MultiBrowser 作为表格的替代方案
        let mut browser = MultiBrowser::new(x, y, w, h, None);
        browser.set_color(Color::White);
        browser.set_selection_color(Color::from_rgb(230, 240, 255));
        browser.set_text_size(12);
        
        // 添加表头
        let header = format!("{:<40} {:<25} {:<6} {:<20} {:<15}", 
            "Title", "Authors", "Year", "Tags", "Category");
        browser.add(&header);
        browser.set_format_char('@');
        browser.add("@-");  // 添加分隔线
        
        container.end();
        
        let files = Arc::new(Mutex::new(Vec::new()));
        
        let mut file_list = Self {
            container,
            browser,
            files,
            selected_index: None,
            event_sender,
        };
        
        file_list.setup_callbacks();
        file_list
    }
    
    fn setup_callbacks(&mut self) {
        let sender = self.event_sender.clone();
        let sender_menu = self.event_sender.clone();
        
        // 选择回调
        self.browser.set_callback(move |browser| {
            let selected = browser.value();
            // 跳过表头（前2行）
            if selected > 2 {
                let _ = sender.send(AppEvent::FileSelected(format!("index:{}", selected - 3)));
            }
        });
        
        // 右键菜单处理
        let files_ref_menu = Arc::clone(&self.files);
        self.browser.handle(move |browser, event| {
            match event {
                Event::Push => {
                    if fltk::app::event_mouse_button() == MouseButton::Right {
                        let selected = browser.value();
                        // 跳过表头
                        if selected > 2 {
                            let files = files_ref_menu.lock().unwrap();
                            let file_index = (selected - 3) as usize;
                            if file_index < files.len() {
                                // 显示右键菜单
                                Self::show_context_menu(file_index, &sender_menu);
                            }
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
        let mut files = self.files.lock().unwrap();
        *files = search_result.entries;
        
        // 清空浏览器（保留表头）
        self.browser.clear();
        
        // 重新添加表头
        let header = format!("{:<40} {:<25} {:<6} {:<20} {:<15}", 
            "Title", "Authors", "Year", "Tags", "Category");
        self.browser.add(&header);
        self.browser.add("@-");  // 分隔线
        
        if files.is_empty() {
            self.browser.add("No files found. Try a different search or import some files.");
            self.browser.deactivate();
            return Ok(());
        }
        
        // 激活浏览器
        self.browser.activate();
        
        // 添加文件数据
        for file in files.iter() {
            // Title
            let display_title = if file.title.is_empty() {
                &file.original_filename
            } else {
                &file.title
            };
            let title = Self::truncate_string(display_title, 40);
            
            // Authors
            let authors_str = if file.authors.is_empty() {
                "Unknown".to_string()
            } else {
                Self::truncate_string(&file.authors.join(", "), 25)
            };
            
            // Year
            let year_str = file.year.map(|y| y.to_string()).unwrap_or_else(|| "----".to_string());
            
            // Tags
            let tags_str = match file.tags.len() {
                0 => "No tags".to_string(),
                1 => file.tags[0].clone(),
                n => format!("{} tags", n),
            };
            
            // Category
            let category_str = if file.category1.is_empty() {
                "Uncategorized".to_string()
            } else {
                Self::truncate_string(&file.category1, 15)
            };
            
            // 格式化行数据
            let line = format!("{:<40} {:<25} {:<6} {:<20} {:<15}", 
                title, authors_str, year_str, tags_str, category_str);
            
            self.browser.add(&line);
        }
        
        println!("Loaded {} files into file list", files.len());
        Ok(())
    }
    
    fn truncate_string(s: &str, max_len: usize) -> String {
        if s.chars().count() > max_len {
            let truncated: String = s.chars().take(max_len - 3).collect();
            format!("{}...", truncated)
        } else {
            s.to_string()
        }
    }
    
    pub fn clear(&mut self) {
        let mut files = self.files.lock().unwrap();
        files.clear();
        self.selected_index = None;
        self.browser.clear();
        
        // 重新添加表头
        let header = format!("{:<40} {:<25} {:<6} {:<20} {:<15}", 
            "Title", "Authors", "Year", "Tags", "Category");
        self.browser.add(&header);
        self.browser.add("@-");
    }
    
    pub fn get_selected_file(&self) -> Option<FileEntry> {
        let selected = self.browser.value();
        if selected > 2 {  // 跳过表头
            let files = self.files.lock().unwrap();
            files.get((selected - 3) as usize).cloned()
        } else {
            None
        }
    }
    
    pub fn select_file(&mut self, file_id: &str) {
        let files = self.files.lock().unwrap();
        for (index, file) in files.iter().enumerate() {
            if file.id == file_id {
                self.selected_index = Some(index);
                self.browser.select(index as i32 + 3); // +3 跳过表头
                println!("Selected file: {} (index: {})", file.title, index);
                break;
            }
        }
    }
    
    // 根据索引选择文件
    pub fn select_file_by_index(&mut self, index: usize) {
        let files = self.files.lock().unwrap();
        if index < files.len() {
            self.selected_index = Some(index);
            self.browser.select(index as i32 + 3); // +3 跳过表头
            if let Some(file) = files.get(index) {
                println!("Selected file by index: {} (index: {})", file.title, index);
            }
        }
    }
    
    // 获取当前选中的文件
    pub fn get_current_selection(&self) -> Option<FileEntry> {
        self.get_selected_file()
    }
    
    // 获取当前文件列表
    pub fn get_current_files(&self) -> Vec<FileEntry> {
        let files = self.files.lock().unwrap();
        files.clone()
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
        let files = self.files.lock().unwrap();
        files.len()
    }
    
    // 获取容器引用（用于主窗口布局）
    pub fn widget(&mut self) -> &mut Group {
        &mut self.container
    }
    
    // 显示拖拽提示
    pub fn show_drag_hint(&mut self, show: bool) {
        if show {
            self.clear();
            self.browser.add("Drag files here to import...");
        } else {
            self.clear();
        }
        self.browser.redraw();
    }
    
    // 显示真正的右键上下文菜单
    fn show_context_menu(file_index: usize, sender: &Sender<AppEvent>) {
        use fltk::menu::*;
        
        let mut menu = MenuButton::default();
        menu.set_pos(fltk::app::event_x(), fltk::app::event_y());
        
        // 创建菜单项
        menu.add_choice("📄 Open File");
        menu.add_choice("✏️ Edit Metadata");
        menu.add_choice("📋 Copy Path");
        menu.add_choice("📁 Show in Folder");
        menu.add_choice("🗑️ Delete");
        
        let choice = menu.popup().map(|item| item.value() as usize);
        
        match choice {
            Some(0) => { // Open File
                let _ = sender.send(AppEvent::OpenFile(format!("index:{}", file_index)));
            },
            Some(1) => { // Edit Metadata
                let _ = sender.send(AppEvent::EditFile(format!("index:{}", file_index)));
            },
            Some(2) => { // Copy Path
                let _ = sender.send(AppEvent::CopyFilePath(format!("index:{}", file_index)));
            },
            Some(3) => { // Show in Folder
                let _ = sender.send(AppEvent::ShowInFolder(format!("index:{}", file_index)));
            },
            Some(4) => { // Delete
                if fltk::dialog::choice2_default("Remove this file from TagBox?", "Cancel", "Remove", "") == Some(1) {
                    let _ = sender.send(AppEvent::DeleteFile(format!("index:{}", file_index)));
                }
            },
            _ => {}
        }
    }
}