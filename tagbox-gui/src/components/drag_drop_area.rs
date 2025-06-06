use fltk::{
    prelude::*,
    group::Group,
    frame::Frame,
    enums::{Color, Align, FrameType},
};
use std::sync::mpsc::Sender;
use crate::state::AppEvent;

pub struct DragDropArea {
    container: Group,
    drop_frame: Frame,
    hint_text: Frame,
    is_active: bool,
    event_sender: Sender<AppEvent>,
}

impl DragDropArea {
    pub fn new(
        x: i32, 
        y: i32, 
        w: i32, 
        h: i32, 
        event_sender: Sender<AppEvent>
    ) -> Self {
        let mut container = Group::new(x, y, w, h, None);
        container.set_color(Color::from_rgb(248, 249, 250));
        
        // 主拖拽区域
        let mut drop_frame = Frame::new(x + 10, y + 10, w - 20, h - 40, None);
        drop_frame.set_frame(FrameType::BorderBox);
        drop_frame.set_color(Color::from_rgb(255, 255, 255));
        drop_frame.set_label_color(Color::from_rgb(108, 117, 125));
        drop_frame.set_align(Align::Center);
        
        // 提示文本
        let mut hint_text = Frame::new(x + 10, y + h - 30, w - 20, 20, 
            "Drag PDF, EPUB, TXT, DOC files here to import");
        hint_text.set_label_color(Color::from_rgb(108, 117, 125));
        hint_text.set_label_size(12);
        hint_text.set_align(Align::Center);
        
        container.end();
        
        let mut drag_drop_area = Self {
            container,
            drop_frame,
            hint_text,
            is_active: false,
            event_sender,
        };
        
        drag_drop_area.setup_drag_drop();
        drag_drop_area.set_default_state();
        drag_drop_area
    }
    
    fn setup_drag_drop(&mut self) {
        let sender = self.event_sender.clone();
        
        self.drop_frame.handle(move |frame, event| {
            match event {
                fltk::enums::Event::DndEnter => {
                    frame.set_color(Color::from_rgb(220, 248, 220));
                    frame.set_label("✓ Drop files here");
                    frame.set_label_color(Color::from_rgb(25, 135, 84));
                    frame.set_label_size(16);
                    frame.redraw();
                    true
                },
                fltk::enums::Event::DndDrag => {
                    true
                },
                fltk::enums::Event::DndLeave => {
                    Self::reset_frame_appearance(frame);
                    false
                },
                fltk::enums::Event::DndRelease => {
                    Self::reset_frame_appearance(frame);
                    
                    // 处理拖拽的文件
                    let file_count = Self::handle_file_drop(&sender);
                    if file_count > 0 {
                        frame.set_color(Color::from_rgb(220, 255, 220));
                        frame.set_label(&format!("✓ {} file(s) imported", file_count));
                        frame.set_label_color(Color::from_rgb(25, 135, 84));
                        frame.redraw();
                        
                        // 3秒后恢复默认状态
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_secs(3));
                            // TODO: 添加重置状态的机制
                        });
                    } else {
                        frame.set_color(Color::from_rgb(255, 220, 220));
                        frame.set_label("✗ No supported files found");
                        frame.set_label_color(Color::from_rgb(220, 53, 69));
                        frame.redraw();
                    }
                    
                    true
                }
                _ => false,
            }
        });
    }
    
    fn reset_frame_appearance(frame: &mut Frame) {
        frame.set_color(Color::from_rgb(255, 255, 255));
        frame.set_label("📁 Drop files here to import");
        frame.set_label_color(Color::from_rgb(108, 117, 125));
        frame.set_label_size(14);
        frame.redraw();
    }
    
    fn handle_file_drop(sender: &Sender<AppEvent>) -> usize {
        let mut imported_count = 0;
        
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
                    if path.exists() {
                        if Self::is_supported_file_type(&path) {
                            let _ = sender.send(AppEvent::FileImport(path));
                            imported_count += 1;
                        } else if path.is_dir() {
                            // 处理文件夹拖拽：递归导入所有支持的文件
                            imported_count += Self::import_directory(&path, sender);
                        }
                    }
                }
            }
        }
        
        imported_count
    }
    
    fn import_directory(dir_path: &std::path::Path, sender: &Sender<AppEvent>) -> usize {
        let mut count = 0;
        
        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && Self::is_supported_file_type(&path) {
                    let _ = sender.send(AppEvent::FileImport(path));
                    count += 1;
                } else if path.is_dir() {
                    // 递归处理子目录
                    count += Self::import_directory(&path, sender);
                }
            }
        }
        
        count
    }
    
    fn is_supported_file_type(path: &std::path::Path) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                matches!(ext_str.to_lowercase().as_str(), 
                    "pdf" | "epub" | "txt" | "md" | "doc" | "docx" | 
                    "rtf" | "html" | "htm" | "odt" | "mobi" | "azw" | "azw3"
                )
            } else {
                false
            }
        } else {
            false
        }
    }
    
    fn set_default_state(&mut self) {
        self.drop_frame.set_label("📁 Drop files here to import");
        self.drop_frame.set_label_color(Color::from_rgb(108, 117, 125));
        self.drop_frame.set_label_size(14);
        self.is_active = false;
    }
    
    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
        if active {
            self.drop_frame.set_color(Color::from_rgb(248, 255, 248));
            self.drop_frame.set_label("🎯 Ready to import files");
            self.drop_frame.set_label_color(Color::from_rgb(25, 135, 84));
            self.hint_text.set_label("Supported: PDF, EPUB, TXT, DOC, DOCX, HTML, RTF, ODT");
        } else {
            self.set_default_state();
            self.hint_text.set_label("Drag PDF, EPUB, TXT, DOC files here to import");
        }
        self.drop_frame.redraw();
        self.hint_text.redraw();
    }
    
    pub fn show_error(&mut self, message: &str) {
        self.drop_frame.set_color(Color::from_rgb(255, 220, 220));
        self.drop_frame.set_label(&format!("✗ {}", message));
        self.drop_frame.set_label_color(Color::from_rgb(220, 53, 69));
        self.drop_frame.redraw();
    }
    
    pub fn show_success(&mut self, message: &str) {
        self.drop_frame.set_color(Color::from_rgb(220, 255, 220));
        self.drop_frame.set_label(&format!("✓ {}", message));
        self.drop_frame.set_label_color(Color::from_rgb(25, 135, 84));
        self.drop_frame.redraw();
    }
    
    pub fn reset(&mut self) {
        self.set_default_state();
        self.drop_frame.redraw();
        self.hint_text.redraw();
    }
    
    pub fn get_supported_extensions() -> &'static [&'static str] {
        &["pdf", "epub", "txt", "md", "doc", "docx", "rtf", "html", "htm", "odt", "mobi", "azw", "azw3"]
    }
    
    pub fn widget(&mut self) -> &mut Group {
        &mut self.container
    }
    
    // 设置拖拽区域大小
    pub fn resize(&mut self, x: i32, y: i32, w: i32, h: i32) {
        self.container.resize(x, y, w, h);
        self.drop_frame.resize(x + 10, y + 10, w - 20, h - 40);
        self.hint_text.resize(x + 10, y + h - 30, w - 20, 20);
    }
    
    // 显示导入统计
    pub fn show_import_stats(&mut self, total: usize, successful: usize, failed: usize) {
        if total == 0 {
            self.show_error("No files to import");
        } else if failed == 0 {
            self.show_success(&format!("Imported {} file(s) successfully", successful));
        } else {
            let message = format!("Imported {}/{} files ({} failed)", successful, total, failed);
            if successful > 0 {
                self.drop_frame.set_color(Color::from_rgb(255, 243, 205));
                self.drop_frame.set_label(&format!("⚠ {}", message));
                self.drop_frame.set_label_color(Color::from_rgb(133, 77, 14));
            } else {
                self.show_error(&format!("Failed to import {} file(s)", failed));
            }
        }
        self.drop_frame.redraw();
    }
}