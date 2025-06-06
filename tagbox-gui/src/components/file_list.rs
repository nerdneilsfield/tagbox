use fltk::{
    prelude::*,
    browser::Browser,
    enums::Color,
    group::Group,
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
        let files_ptr = &self.files as *const Vec<FileEntry>;
        
        self.browser.set_callback(move |browser| {
            let selected = browser.value();
            if selected > 0 {
                let files = unsafe { &*files_ptr };
                if let Some(file) = files.get((selected - 1) as usize) {
                    let _ = sender.send(AppEvent::FileSelected(file.id.clone()));
                }
            }
        });
    }
    
    pub async fn load_files(&mut self, search_result: SearchResult) -> Result<(), Box<dyn std::error::Error>> {
        self.files = search_result.entries;
        
        // 清空浏览器
        self.browser.clear();
        
        // 添加文件到浏览器
        for file in &self.files {
            let display_title = if file.title.is_empty() {
                &file.original_filename
            } else {
                &file.title
            };
            
            let authors_str = if file.authors.is_empty() {
                "Unknown"
            } else {
                &file.authors.join(", ")
            };
            
            let year_str = file.year.map(|y| y.to_string()).unwrap_or_else(|| "N/A".to_string());
            
            let tags_str = if file.tags.is_empty() {
                "No tags"
            } else {
                &file.tags.join(", ")
            };
            
            let line = format!("{} | {} | {} | {}", 
                display_title, authors_str, year_str, tags_str);
            
            self.browser.add(&line);
        }
        
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
                break;
            }
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
}