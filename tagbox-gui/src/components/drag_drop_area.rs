use fltk::{
    prelude::*,
    group::Group,
    frame::Frame,
    enums::{Color, Align, FrameType},
};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::state::AppEvent;

pub struct DragDropArea {
    container: Group,
    drop_frame: Frame,
    hint_text: Frame,
    progress_frame: Frame,
    is_active: bool,
    is_dragging: bool,
    last_feedback_time: Option<Instant>,
    import_stats: Arc<Mutex<ImportStats>>,
    event_sender: Sender<AppEvent>,
}

#[derive(Debug, Clone, Default)]
struct ImportStats {
    total_files: usize,
    successful: usize,
    failed: usize,
    current_file: Option<String>,
    is_importing: bool,
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
        let mut hint_text = Frame::new(x + 10, y + h - 50, w - 20, 20, 
            "Drag PDF, EPUB, TXT, DOC files here to import");
        hint_text.set_label_color(Color::from_rgb(108, 117, 125));
        hint_text.set_label_size(12);
        hint_text.set_align(Align::Center);
        
        // 进度显示框
        let mut progress_frame = Frame::new(x + 10, y + h - 25, w - 20, 15, None);
        progress_frame.set_label_color(Color::from_rgb(25, 135, 84));
        progress_frame.set_label_size(10);
        progress_frame.set_align(Align::Center);
        progress_frame.hide();
        
        container.end();
        
        let mut drag_drop_area = Self {
            container,
            drop_frame,
            hint_text,
            progress_frame,
            is_active: false,
            is_dragging: false,
            last_feedback_time: None,
            import_stats: Arc::new(Mutex::new(ImportStats::default())),
            event_sender,
        };
        
        drag_drop_area.setup_drag_drop();
        drag_drop_area.set_default_state();
        drag_drop_area
    }
    
    fn setup_drag_drop(&mut self) {
        let sender = self.event_sender.clone();
        let stats = self.import_stats.clone();
        
        self.drop_frame.handle(move |frame, event| {
            match event {
                fltk::enums::Event::DndEnter => {
                    // 增强的拖拽进入效果
                    frame.set_color(Color::from_rgb(220, 248, 220));
                    frame.set_label("🎯 Drop files here to import");
                    frame.set_label_color(Color::from_rgb(25, 135, 84));
                    frame.set_label_size(16);
                    frame.set_frame(FrameType::BorderBox);
                    frame.redraw();
                    true
                },
                fltk::enums::Event::DndDrag => {
                    // 拖拽过程中的动态反馈
                    let now = Instant::now();
                    // 每500ms更新一次拖拽提示
                    if now.duration_since(Instant::now()).as_millis() % 500 < 100 {
                        frame.set_label("📥 Ready to import...");
                        frame.redraw();
                    }
                    true
                },
                fltk::enums::Event::DndLeave => {
                    Self::reset_frame_appearance(frame);
                    false
                },
                fltk::enums::Event::DndRelease => {
                    // 显示正在处理状态
                    frame.set_color(Color::from_rgb(255, 248, 220));
                    frame.set_label("⏳ Processing files...");
                    frame.set_label_color(Color::from_rgb(133, 77, 14));
                    frame.redraw();
                    
                    // 处理拖拽的文件
                    let (file_count, supported_count) = Self::handle_file_drop_enhanced(&sender, &stats);
                    
                    // 立即显示结果（不使用线程以避免生命周期问题）
                    if supported_count > 0 {
                        frame.set_color(Color::from_rgb(220, 255, 220));
                        if file_count == supported_count {
                            frame.set_label(&format!("✅ {} file(s) imported successfully", supported_count));
                        } else {
                            frame.set_label(&format!("⚠️ {}/{} files imported", supported_count, file_count));
                        }
                        frame.set_label_color(Color::from_rgb(25, 135, 84));
                    } else if file_count > 0 {
                        frame.set_color(Color::from_rgb(255, 220, 220));
                        frame.set_label(&format!("❌ No supported files in {} item(s)", file_count));
                        frame.set_label_color(Color::from_rgb(220, 53, 69));
                    } else {
                        frame.set_color(Color::from_rgb(255, 220, 220));
                        frame.set_label("❌ No files found");
                        frame.set_label_color(Color::from_rgb(220, 53, 69));
                    }
                    frame.redraw();
                    
                    // TODO: 在主事件循环中实现延迟重置
                    
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
    
    fn handle_file_drop_enhanced(sender: &Sender<AppEvent>, stats: &Arc<Mutex<ImportStats>>) -> (usize, usize) {
        let mut total_count = 0;
        let mut imported_count = 0;
        
        // 重置统计信息
        if let Ok(mut stats_guard) = stats.lock() {
            *stats_guard = ImportStats::default();
            stats_guard.is_importing = true;
        }
        
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
                        total_count += 1;
                        
                        if Self::is_supported_file_type(&path) {
                            // 更新当前处理的文件
                            if let Ok(mut stats_guard) = stats.lock() {
                                stats_guard.current_file = path.file_name()
                                    .and_then(|n| n.to_str())
                                    .map(|s| s.to_string());
                            }
                            
                            let _ = sender.send(AppEvent::FileImport(path));
                            imported_count += 1;
                        } else if path.is_dir() {
                            // 处理文件夹拖拽：递归导入所有支持的文件
                            let dir_count = Self::import_directory_enhanced(&path, sender, stats);
                            imported_count += dir_count;
                            total_count += dir_count; // 目录中的文件也计入总数
                        }
                    }
                }
            }
        }
        
        // 完成导入统计
        if let Ok(mut stats_guard) = stats.lock() {
            stats_guard.total_files = total_count;
            stats_guard.successful = imported_count;
            stats_guard.failed = total_count - imported_count;
            stats_guard.is_importing = false;
            stats_guard.current_file = None;
        }
        
        (total_count, imported_count)
    }
    
    fn handle_file_drop(sender: &Sender<AppEvent>) -> usize {
        let stats = Arc::new(Mutex::new(ImportStats::default()));
        let (_, imported) = Self::handle_file_drop_enhanced(sender, &stats);
        imported
    }
    
    fn import_directory_enhanced(dir_path: &std::path::Path, sender: &Sender<AppEvent>, stats: &Arc<Mutex<ImportStats>>) -> usize {
        let mut count = 0;
        
        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && Self::is_supported_file_type(&path) {
                    // 更新当前处理的文件
                    if let Ok(mut stats_guard) = stats.lock() {
                        stats_guard.current_file = path.file_name()
                            .and_then(|n| n.to_str())
                            .map(|s| s.to_string());
                    }
                    
                    let _ = sender.send(AppEvent::FileImport(path));
                    count += 1;
                } else if path.is_dir() {
                    // 递归处理子目录
                    count += Self::import_directory_enhanced(&path, sender, stats);
                }
            }
        }
        
        count
    }
    
    fn import_directory(dir_path: &std::path::Path, sender: &Sender<AppEvent>) -> usize {
        let stats = Arc::new(Mutex::new(ImportStats::default()));
        Self::import_directory_enhanced(dir_path, sender, &stats)
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
        self.drop_frame.resize(x + 10, y + 10, w - 20, h - 60);
        self.hint_text.resize(x + 10, y + h - 50, w - 20, 20);
        self.progress_frame.resize(x + 10, y + h - 25, w - 20, 15);
    }
    
    // 显示导入统计（增强版）
    pub fn show_import_stats(&mut self, total: usize, successful: usize, failed: usize) {
        // 更新内部统计
        if let Ok(mut stats) = self.import_stats.lock() {
            stats.total_files = total;
            stats.successful = successful;
            stats.failed = failed;
            stats.is_importing = false;
        }
        
        if total == 0 {
            self.show_error("No files to import");
            self.progress_frame.hide();
        } else if failed == 0 {
            self.show_success(&format!("✅ Imported {} file(s) successfully", successful));
            self.show_progress_info(&format!("All {} files processed successfully", total));
        } else {
            let message = format!("Imported {}/{} files ({} failed)", successful, total, failed);
            if successful > 0 {
                self.drop_frame.set_color(Color::from_rgb(255, 243, 205));
                self.drop_frame.set_label(&format!("⚠️ {}", message));
                self.drop_frame.set_label_color(Color::from_rgb(133, 77, 14));
                self.show_progress_info(&format!("Processed: {} success, {} failed", successful, failed));
            } else {
                self.show_error(&format!("❌ Failed to import {} file(s)", failed));
                self.progress_frame.hide();
            }
        }
        self.drop_frame.redraw();
        
        // TODO: 在主事件循环中实现延迟重置（不使用unsafe代码）
    }
    
    // 显示进度信息
    pub fn show_progress_info(&mut self, message: &str) {
        self.progress_frame.set_label(message);
        self.progress_frame.show();
        self.progress_frame.redraw();
    }
    
    // 显示实时导入进度
    pub fn show_import_progress(&mut self, current: usize, total: usize, current_file: Option<&str>) {
        let progress_text = if let Some(filename) = current_file {
            format!("Importing {} ({}/{})", filename, current, total)
        } else {
            format!("Processing files... ({}/{})", current, total)
        };
        
        self.show_progress_info(&progress_text);
        
        // 更新主区域显示
        self.drop_frame.set_color(Color::from_rgb(255, 248, 220));
        self.drop_frame.set_label(&format!("⏳ Importing files... {:.0}%", (current as f32 / total as f32) * 100.0));
        self.drop_frame.set_label_color(Color::from_rgb(133, 77, 14));
        self.drop_frame.redraw();
    }
    
    // 获取导入统计信息
    pub fn get_import_stats(&self) -> Option<ImportStats> {
        self.import_stats.lock().ok().map(|stats| stats.clone())
    }
    
    // 检查是否正在导入
    pub fn is_importing(&self) -> bool {
        self.import_stats.lock().ok()
            .map(|stats| stats.is_importing)
            .unwrap_or(false)
    }
}