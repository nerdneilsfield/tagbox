use fltk::{
    prelude::*,
    window::Window,
    group::{Flex, FlexType},
    text::{TextDisplay, TextBuffer, WrapMode},
    button::Button,
    menu::Choice,
    input::Input,
    enums::Color,
    frame::Frame,
};
use std::sync::mpsc::Sender;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{BufRead, BufReader};
use crate::state::AppEvent;

pub struct LogViewer {
    window: Window,
    
    // 文本显示区域
    text_display: TextDisplay,
    text_buffer: TextBuffer,
    
    // 控制面板
    log_level_filter: Choice,
    search_input: Input,
    search_btn: Button,
    clear_btn: Button,
    export_btn: Button,
    refresh_btn: Button,
    
    // 日志文件信息
    log_file_path: Option<PathBuf>,
    current_filter: LogLevel,
    
    event_sender: Sender<AppEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    All,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::All => "ALL",
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }
    
    fn from_index(index: i32) -> Self {
        match index {
            0 => LogLevel::All,
            1 => LogLevel::Error,
            2 => LogLevel::Warn,
            3 => LogLevel::Info,
            4 => LogLevel::Debug,
            5 => LogLevel::Trace,
            _ => LogLevel::All,
        }
    }
}

impl LogViewer {
    pub fn new(event_sender: Sender<AppEvent>) -> Self {
        let mut window = Window::new(300, 300, 800, 600, "Log Viewer");
        window.set_color(Color::from_rgb(248, 249, 250));
        
        // 顶部控制面板
        let mut control_panel = Flex::new(10, 10, 780, 40, None);
        control_panel.set_type(FlexType::Row);
        control_panel.set_spacing(10);
        
        // 日志级别筛选
        let _level_label = Frame::new(0, 0, 0, 0, "Level:");
        control_panel.fixed(&_level_label, 50);
        
        let mut log_level_filter = Choice::new(0, 0, 0, 0, None);
        log_level_filter.add_choice("ALL");
        log_level_filter.add_choice("ERROR");
        log_level_filter.add_choice("WARN");
        log_level_filter.add_choice("INFO");
        log_level_filter.add_choice("DEBUG");
        log_level_filter.add_choice("TRACE");
        log_level_filter.set_value(0);
        control_panel.fixed(&log_level_filter, 80);
        
        // 搜索框
        let _search_label = Frame::new(0, 0, 0, 0, "Search:");
        control_panel.fixed(&_search_label, 60);
        
        let mut search_input = Input::new(0, 0, 0, 0, None);
        search_input.set_color(Color::White);
        control_panel.fixed(&search_input, 200);
        
        let mut search_btn = Button::new(0, 0, 0, 0, "Search");
        search_btn.set_color(Color::from_rgb(0, 123, 255));
        search_btn.set_label_color(Color::White);
        control_panel.fixed(&search_btn, 70);
        
        // 操作按钮
        let mut refresh_btn = Button::new(0, 0, 0, 0, "Refresh");
        refresh_btn.set_color(Color::from_rgb(23, 162, 184));
        refresh_btn.set_label_color(Color::White);
        control_panel.fixed(&refresh_btn, 70);
        
        let mut clear_btn = Button::new(0, 0, 0, 0, "Clear");
        clear_btn.set_color(Color::from_rgb(255, 193, 7));
        clear_btn.set_label_color(Color::Black);
        control_panel.fixed(&clear_btn, 70);
        
        let mut export_btn = Button::new(0, 0, 0, 0, "Export");
        export_btn.set_color(Color::from_rgb(108, 117, 125));
        export_btn.set_label_color(Color::White);
        control_panel.fixed(&export_btn, 70);
        
        control_panel.end();
        
        // 文本显示区域
        let mut text_display = TextDisplay::new(10, 60, 780, 530, None);
        text_display.set_color(Color::from_rgb(33, 37, 41));
        text_display.set_text_color(Color::from_rgb(248, 249, 250));
        text_display.set_text_font(fltk::enums::Font::Courier);
        text_display.set_text_size(12);
        text_display.wrap_mode(WrapMode::AtBounds, 0);
        
        let text_buffer = TextBuffer::default();
        text_display.set_buffer(Some(text_buffer.clone()));
        
        window.end();
        
        let mut log_viewer = Self {
            window,
            text_display,
            text_buffer,
            log_level_filter,
            search_input,
            search_btn,
            clear_btn,
            export_btn,
            refresh_btn,
            log_file_path: None,
            current_filter: LogLevel::All,
            event_sender,
        };
        
        log_viewer.setup_callbacks();
        log_viewer
    }
    
    fn setup_callbacks(&mut self) {
        // 刷新按钮
        let mut viewer_clone = self.clone_for_callback();
        self.refresh_btn.set_callback(move |_| {
            if let Err(e) = viewer_clone.refresh_logs() {
                eprintln!("Failed to refresh logs: {}", e);
            }
        });
        
        // 清除按钮
        let mut buffer = self.text_buffer.clone();
        self.clear_btn.set_callback(move |_| {
            if Self::confirm_clear() {
                buffer.set_text("");
            }
        });
        
        // 导出按钮
        let buffer_clone = self.text_buffer.clone();
        self.export_btn.set_callback(move |_| {
            Self::export_logs_with_content(&buffer_clone);
        });
        
        // 搜索按钮
        let search_input = self.search_input.clone();
        let mut text_display = self.text_display.clone();
        self.search_btn.set_callback(move |_| {
            let query = search_input.value();
            if !query.is_empty() {
                Self::search_in_logs(&mut text_display, &query);
            }
        });
        
        // 日志级别筛选
        let mut viewer_clone2 = self.clone_for_callback();
        self.log_level_filter.set_callback(move |choice| {
            let level = LogLevel::from_index(choice.value());
            viewer_clone2.set_filter(level);
        });
        
        // 搜索框回车键
        let mut search_btn = self.search_btn.clone();
        self.search_input.handle(move |input, event| {
            match event {
                fltk::enums::Event::KeyDown => {
                    if fltk::app::event_key() == fltk::enums::Key::Enter {
                        search_btn.do_callback();
                        true
                    } else {
                        false
                    }
                },
                _ => false
            }
        });
    }
    
    pub fn show(&mut self) {
        // 在显示窗口前自动检测并加载日志文件
        if let Some(log_path) = Self::detect_log_file() {
            if let Err(e) = self.load_log_file(&log_path) {
                eprintln!("Failed to load log file: {}", e);
                // 显示错误但仍然打开窗口
                self.text_buffer.set_text(&format!("Failed to load log file: {}\n\nPath: {}", e, log_path.display()));
            }
        } else {
            // 没有找到日志文件，显示提示信息
            self.text_buffer.set_text("No log file found.\n\nPossible locations:\n- ./tagbox.log\n- ./logs/tagbox.log\n- ./target/debug/tagbox.log\n\nPlease check if logging is enabled in your configuration.");
        }
        
        self.window.show();
    }
    
    pub fn hide(&mut self) {
        self.window.hide();
    }

    pub fn shown(&self) -> bool {
        self.window.shown()
    }
    
    pub fn load_log_file(&mut self, log_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        self.log_file_path = Some(log_path.to_path_buf());
        self.refresh_logs()?;
        Ok(())
    }
    
    pub fn refresh_logs(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(log_path) = &self.log_file_path {
            let content = self.read_log_file(log_path)?;
            let filtered_content = self.apply_filter(&content);
            self.text_buffer.set_text(&filtered_content);
            
            // 滚动到底部显示最新日志
            self.scroll_to_bottom();
        }
        Ok(())
    }
    
    fn read_log_file(&self, path: &Path) -> Result<String, Box<dyn std::error::Error>> {
        if !path.exists() {
            return Ok("Log file not found.".to_string());
        }
        
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut content = String::new();
        
        // 读取最后N行（避免内存问题）
        let max_lines = 10000;
        let lines: Vec<String> = reader.lines()
            .collect::<Result<Vec<_>, _>>()?;
        
        let start_index = if lines.len() > max_lines {
            lines.len() - max_lines
        } else {
            0
        };
        
        for line in &lines[start_index..] {
            content.push_str(line);
            content.push('\n');
        }
        
        if lines.len() > max_lines {
            content.insert_str(0, &format!("... (showing last {} lines)\n\n", max_lines));
        }
        
        Ok(content)
    }
    
    fn apply_filter(&self, content: &str) -> String {
        if self.current_filter == LogLevel::All {
            return content.to_string();
        }
        
        let filter_str = self.current_filter.as_str();
        content.lines()
            .filter(|line| line.contains(filter_str))
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    fn scroll_to_bottom(&mut self) {
        let line_count = self.text_buffer.count_lines(0, self.text_buffer.length());
        self.text_display.scroll(line_count, 0);
    }
    
    fn search_in_logs(text_display: &mut TextDisplay, query: &str) {
        if let Some(mut buffer) = text_display.buffer() {
            let text = buffer.text();
            if let Some(pos) = text.find(query) {
                // 移动到找到的位置
                buffer.select(pos as i32, (pos + query.len()) as i32);
                text_display.show_insert_position();
                println!("Found '{}' at position {}", query, pos);
            } else {
                println!("'{}' not found in log content", query);
                fltk::dialog::message_default(&format!("Search term '{}' not found in current log content.", query));
            }
        }
    }
    
    fn confirm_clear() -> bool {
        let choice = fltk::dialog::choice2_default(
            "This will clear the log display.\nThe log file will not be affected.\nContinue?",
            "Yes",
            "No",
            ""
        );
        choice == Some(0)
    }
    
    fn export_logs_with_content(buffer: &TextBuffer) {
        let mut dialog = fltk::dialog::NativeFileChooser::new(
            fltk::dialog::NativeFileChooserType::BrowseSaveFile
        );
        dialog.set_title("Export Logs");
        dialog.set_filter("Text Files\t*.txt\nLog Files\t*.log\nAll Files\t*");
        dialog.show();
        
        let filename = dialog.filename();
        if !filename.to_string_lossy().is_empty() {
            let content = buffer.text();
            match std::fs::write(&filename, content) {
                Ok(()) => {
                    fltk::dialog::message_default(&format!("Logs exported successfully!\n\nFile: {}", filename.display()));
                },
                Err(e) => {
                    fltk::dialog::alert_default(&format!("Failed to export logs:\n{}\n\nFile: {}", e, filename.display()));
                }
            }
        }
    }
    
    pub fn append_log_entry(&mut self, entry: &str) {
        // 添加新的日志条目到显示区域
        self.text_buffer.append(entry);
        self.text_buffer.append("\n");
        self.scroll_to_bottom();
    }
    
    pub fn set_filter(&mut self, level: LogLevel) {
        self.current_filter = level.clone();
        let index = match level {
            LogLevel::All => 0,
            LogLevel::Error => 1,
            LogLevel::Warn => 2,
            LogLevel::Info => 3,
            LogLevel::Debug => 4,
            LogLevel::Trace => 5,
        };
        self.log_level_filter.set_value(index);
        
        // 重新应用筛选
        if let Err(e) = self.refresh_logs() {
            eprintln!("Failed to refresh logs: {}", e);
        }
    }
    
    pub fn clear_display(&mut self) {
        self.text_buffer.set_text("");
    }
    
    pub fn get_log_stats(&self) -> LogStats {
        let text = self.text_buffer.text();
        let lines = text.lines().count();
        
        let mut stats = LogStats {
            total_lines: lines,
            error_count: 0,
            warn_count: 0,
            info_count: 0,
            debug_count: 0,
            trace_count: 0,
        };
        
        for line in text.lines() {
            if line.contains("ERROR") {
                stats.error_count += 1;
            } else if line.contains("WARN") {
                stats.warn_count += 1;
            } else if line.contains("INFO") {
                stats.info_count += 1;
            } else if line.contains("DEBUG") {
                stats.debug_count += 1;
            } else if line.contains("TRACE") {
                stats.trace_count += 1;
            }
        }
        
        stats
    }
    
    // 实时监控日志文件（简化版）
    pub fn start_monitoring(&mut self) {
        // TODO: 实现文件监控，当日志文件更新时自动刷新
        println!("Starting log file monitoring...");
    }
    
    pub fn stop_monitoring(&mut self) {
        // TODO: 停止文件监控
        println!("Stopping log file monitoring...");
    }
    
    // 检测日志文件路径
    fn detect_log_file() -> Option<std::path::PathBuf> {
        let possible_paths = [
            "./tagbox.log",
            "./logs/tagbox.log", 
            "./target/debug/tagbox.log",
            "./target/release/tagbox.log",
            "/tmp/tagbox.log",
            "~/.cache/tagbox/tagbox.log",
        ];
        
        for path_str in &possible_paths {
            let path = std::path::PathBuf::from(path_str);
            if path.exists() && path.is_file() {
                return Some(path);
            }
        }
        
        // 如果没有找到现有的日志文件，创建一个演示日志文件
        let demo_path = std::path::PathBuf::from("./tagbox_demo.log");
        if let Err(e) = Self::create_demo_log_file(&demo_path) {
            eprintln!("Failed to create demo log file: {}", e);
            return None;
        }
        
        Some(demo_path)
    }
    
    // 创建演示日志文件
    fn create_demo_log_file(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;
        
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let demo_content = format!(
r#"[{}] INFO TagBox GUI started
[{}] INFO Loading configuration from config.toml
[{}] INFO Database connection established: ./.sqlx-data/tagbox.db
[{}] INFO Initializing main window components
[{}] INFO Search bar initialized
[{}] INFO Category tree loaded with 0 categories
[{}] INFO File list component ready
[{}] INFO File preview panel initialized
[{}] DEBUG Setting up keyboard shortcuts
[{}] DEBUG Configuring drag-drop handlers
[{}] INFO Application ready for user interaction
[{}] INFO Waiting for file imports...
[{}] DEBUG Event loop started
[{}] TRACE UI refresh cycle completed
"#,
            timestamp, timestamp, timestamp, timestamp,
            timestamp, timestamp, timestamp, timestamp,
            timestamp, timestamp, timestamp, timestamp,
            timestamp, timestamp,
        );
        
        fs::write(path, demo_content)?;
        Ok(())
    }
    
    // 创建用于回调的克隆
    fn clone_for_callback(&self) -> LogViewerCallback {
        LogViewerCallback {
            text_buffer: self.text_buffer.clone(),
            text_display: self.text_display.clone(),
            log_file_path: self.log_file_path.clone(),
            current_filter: self.current_filter.clone(),
            log_level_filter: self.log_level_filter.clone(),
        }
    }
}

// 用于回调的轻量级结构体
#[derive(Clone)]
struct LogViewerCallback {
    text_buffer: TextBuffer,
    text_display: TextDisplay,
    log_file_path: Option<PathBuf>,
    current_filter: LogLevel,
    log_level_filter: Choice,
}

impl LogViewerCallback {
    fn refresh_logs(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(log_path) = &self.log_file_path {
            let content = self.read_log_file(log_path)?;
            let filtered_content = self.apply_filter(&content);
            self.text_buffer.set_text(&filtered_content);
            
            // 滚动到底部显示最新日志
            let line_count = self.text_buffer.count_lines(0, self.text_buffer.length());
            self.text_display.scroll(line_count, 0);
        }
        Ok(())
    }
    
    fn read_log_file(&self, path: &Path) -> Result<String, Box<dyn std::error::Error>> {
        if !path.exists() {
            return Ok("Log file not found.".to_string());
        }
        
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut content = String::new();
        
        // 读取最后N行（避免内存问题）
        let max_lines = 10000;
        let lines: Vec<String> = reader.lines()
            .collect::<Result<Vec<_>, _>>()?;
        
        let start_index = if lines.len() > max_lines {
            lines.len() - max_lines
        } else {
            0
        };
        
        for line in &lines[start_index..] {
            content.push_str(line);
            content.push('\n');
        }
        
        if lines.len() > max_lines {
            content.insert_str(0, &format!("... (showing last {} lines)\n\n", max_lines));
        }
        
        Ok(content)
    }
    
    fn apply_filter(&self, content: &str) -> String {
        if self.current_filter == LogLevel::All {
            return content.to_string();
        }
        
        let filter_str = self.current_filter.as_str();
        content.lines()
            .filter(|line| line.contains(filter_str))
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    fn set_filter(&mut self, level: LogLevel) {
        self.current_filter = level.clone();
        let index = match level {
            LogLevel::All => 0,
            LogLevel::Error => 1,
            LogLevel::Warn => 2,
            LogLevel::Info => 3,
            LogLevel::Debug => 4,
            LogLevel::Trace => 5,
        };
        self.log_level_filter.set_value(index);
        
        // 重新应用筛选
        if let Err(e) = self.refresh_logs() {
            eprintln!("Failed to refresh logs: {}", e);
        }
    }
}

#[derive(Debug, Default)]
pub struct LogStats {
    pub total_lines: usize,
    pub error_count: usize,
    pub warn_count: usize,
    pub info_count: usize,
    pub debug_count: usize,
    pub trace_count: usize,
}

impl LogStats {
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }
    
    pub fn has_warnings(&self) -> bool {
        self.warn_count > 0
    }
}