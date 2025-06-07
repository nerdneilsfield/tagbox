use fltk::{
    prelude::*,
    group::{Group, Flex, FlexType},
    output::Output,
    button::Button,
    enums::{Color, Align},
    frame::Frame,
};
use std::sync::mpsc::Sender;
use std::path::PathBuf;
use crate::state::AppEvent;

pub struct StatusBar {
    container: Group,
    
    // 状态信息显示
    status_message: Output,
    file_count: Output,
    selected_count: Output,
    
    // 配置信息
    config_status: Output,
    config_path_btn: Button,
    
    // 日志信息
    log_level: Output,
    log_status: Button,
    
    // 数据库状态
    db_status: Output,
    db_size: Output,
    
    // 搜索状态
    search_status: Output,
    
    // 进度信息
    progress_frame: Frame,
    
    event_sender: Sender<AppEvent>,
}

impl StatusBar {
    pub fn new(x: i32, y: i32, w: i32, h: i32, event_sender: Sender<AppEvent>) -> Self {
        let mut container = Group::new(x, y, w, h, None);
        container.set_color(Color::from_rgb(240, 240, 240));
        
        let mut flex = Flex::new(x, y, w, h, None);
        flex.set_type(FlexType::Row);
        flex.set_spacing(10);
        
        // 状态消息（主要区域）
        let mut status_message = Output::new(0, 0, 0, 0, None);
        status_message.set_value("Ready");
        status_message.set_color(Color::from_rgb(240, 240, 240));
        status_message.set_text_color(Color::Black);
        status_message.set_align(Align::Left | Align::Inside);
        flex.fixed(&status_message, 200);
        
        // 文件计数
        let mut file_count = Output::new(0, 0, 0, 0, None);
        file_count.set_value("Files: 0");
        file_count.set_color(Color::from_rgb(240, 240, 240));
        file_count.set_text_color(Color::Black);
        file_count.set_align(Align::Center);
        flex.fixed(&file_count, 80);
        
        // 选中文件计数
        let mut selected_count = Output::new(0, 0, 0, 0, None);
        selected_count.set_value("Selected: 0");
        selected_count.set_color(Color::from_rgb(240, 240, 240));
        selected_count.set_text_color(Color::Black);
        selected_count.set_align(Align::Center);
        flex.fixed(&selected_count, 90);
        
        // 配置状态
        let mut config_status = Output::new(0, 0, 0, 0, None);
        config_status.set_value("Config: None");
        config_status.set_color(Color::from_rgb(240, 240, 240));
        config_status.set_text_color(Color::from_rgb(200, 100, 100));
        config_status.set_align(Align::Center);
        flex.fixed(&config_status, 100);
        
        // 配置路径按钮
        let mut config_path_btn = Button::new(0, 0, 0, 0, "⚙");
        config_path_btn.set_color(Color::from_rgb(240, 240, 240));
        config_path_btn.set_tooltip("Click to view/edit configuration");
        flex.fixed(&config_path_btn, 25);
        
        // 日志级别
        let mut log_level = Output::new(0, 0, 0, 0, None);
        log_level.set_value("Log: INFO");
        log_level.set_color(Color::from_rgb(240, 240, 240));
        log_level.set_text_color(Color::Black);
        log_level.set_align(Align::Center);
        flex.fixed(&log_level, 80);
        
        // 日志状态按钮
        let mut log_status = Button::new(0, 0, 0, 0, "📋");
        log_status.set_color(Color::from_rgb(240, 240, 240));
        log_status.set_tooltip("Click to view logs");
        flex.fixed(&log_status, 25);
        
        // 数据库状态
        let mut db_status = Output::new(0, 0, 0, 0, None);
        db_status.set_value("DB: Connected");
        db_status.set_color(Color::from_rgb(240, 240, 240));
        db_status.set_text_color(Color::from_rgb(100, 200, 100));
        db_status.set_align(Align::Center);
        flex.fixed(&db_status, 100);
        
        // 数据库大小
        let mut db_size = Output::new(0, 0, 0, 0, None);
        db_size.set_value("0 MB");
        db_size.set_color(Color::from_rgb(240, 240, 240));
        db_size.set_text_color(Color::Black);
        db_size.set_align(Align::Center);
        flex.fixed(&db_size, 60);
        
        // 搜索状态
        let mut search_status = Output::new(0, 0, 0, 0, None);
        search_status.set_value("Search: Ready");
        search_status.set_color(Color::from_rgb(240, 240, 240));
        search_status.set_text_color(Color::Black);
        search_status.set_align(Align::Center);
        flex.fixed(&search_status, 100);
        
        // 进度指示器
        let mut progress_frame = Frame::new(0, 0, 0, 0, None);
        progress_frame.set_color(Color::from_rgb(240, 240, 240));
        flex.fixed(&progress_frame, 50);
        
        flex.end();
        container.end();
        
        let mut status_bar = Self {
            container,
            status_message,
            file_count,
            selected_count,
            config_status,
            config_path_btn,
            log_level,
            log_status,
            db_status,
            db_size,
            search_status,
            progress_frame,
            event_sender,
        };
        
        status_bar.setup_callbacks();
        status_bar
    }
    
    fn setup_callbacks(&mut self) {
        // 配置按钮回调
        let sender = self.event_sender.clone();
        self.config_path_btn.set_callback(move |_| {
            // TODO: 打开配置设置对话框
            println!("Opening configuration settings...");
        });
        
        // 日志按钮回调
        let sender = self.event_sender.clone();
        self.log_status.set_callback(move |_| {
            // TODO: 打开日志查看器
            println!("Opening log viewer...");
        });
    }
    
    // 更新状态消息
    pub fn set_status(&mut self, message: &str) {
        self.status_message.set_value(message);
        self.status_message.redraw();
    }
    
    // 设置消息（带错误标志）
    pub fn set_message(&mut self, message: &str, is_error: bool) {
        self.status_message.set_value(message);
        
        // 根据是否为错误设置不同颜色
        if is_error {
            self.status_message.set_text_color(Color::from_rgb(220, 53, 69)); // 红色
        } else {
            self.status_message.set_text_color(Color::from_rgb(25, 135, 84)); // 绿色
        }
        
        self.status_message.redraw();
        
        // 3秒后恢复默认颜色
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(3));
            // 注意：实际应用中需要通过事件系统来重置颜色
        });
    }
    
    // 设置临时状态消息（会自动清除）
    pub fn set_temp_status(&mut self, message: &str, duration_ms: u64) {
        self.set_status(message);
        
        // TODO: 实现定时器来清除状态
        // 现在简单设置一个标记
        println!("Temp status: {} (duration: {}ms)", message, duration_ms);
    }
    
    // 更新文件计数
    pub fn set_file_count(&mut self, total: usize, visible: Option<usize>) {
        let text = if let Some(vis) = visible {
            format!("Files: {}/{}", vis, total)
        } else {
            format!("Files: {}", total)
        };
        self.file_count.set_value(&text);
        self.file_count.redraw();
    }
    
    // 更新选中文件计数
    pub fn set_selected_count(&mut self, count: usize) {
        self.selected_count.set_value(&format!("Selected: {}", count));
        self.selected_count.redraw();
    }
    
    // 更新配置状态
    pub fn set_config_status(&mut self, config_path: Option<&PathBuf>) {
        match config_path {
            Some(path) => {
                let filename = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("config.toml");
                self.config_status.set_value(&format!("Config: {}", filename));
                self.config_status.set_text_color(Color::from_rgb(100, 200, 100));
                self.config_path_btn.set_tooltip(&format!("Configuration: {}", path.display()));
            },
            None => {
                self.config_status.set_value("Config: None");
                self.config_status.set_text_color(Color::from_rgb(200, 100, 100));
                self.config_path_btn.set_tooltip("No configuration file loaded");
            }
        }
        self.config_status.redraw();
    }
    
    // 更新日志级别
    pub fn set_log_level(&mut self, level: &str) {
        self.log_level.set_value(&format!("Log: {}", level));
        
        // 根据日志级别设置颜色
        let color = match level.to_uppercase().as_str() {
            "ERROR" => Color::from_rgb(220, 53, 69),
            "WARN" => Color::from_rgb(255, 193, 7),
            "INFO" => Color::from_rgb(23, 162, 184),
            "DEBUG" => Color::from_rgb(108, 117, 125),
            "TRACE" => Color::from_rgb(108, 117, 125),
            _ => Color::Black,
        };
        
        self.log_level.set_text_color(color);
        self.log_level.redraw();
    }
    
    // 更新数据库状态
    pub fn set_database_status(&mut self, connected: bool, size_mb: Option<f64>) {
        if connected {
            self.db_status.set_value("DB: Connected");
            self.db_status.set_text_color(Color::from_rgb(100, 200, 100));
        } else {
            self.db_status.set_value("DB: Disconnected");
            self.db_status.set_text_color(Color::from_rgb(200, 100, 100));
        }
        
        if let Some(size) = size_mb {
            self.db_size.set_value(&format!("{:.1} MB", size));
        } else {
            self.db_size.set_value("-- MB");
        }
        
        self.db_status.redraw();
        self.db_size.redraw();
    }
    
    // 更新搜索状态
    pub fn set_search_status(&mut self, status: SearchStatus) {
        let (text, color) = match status {
            SearchStatus::Ready => ("Search: Ready", Color::Black),
            SearchStatus::Searching => ("Search: Searching...", Color::from_rgb(23, 162, 184)),
            SearchStatus::Results(count) => {
                let text: &'static str = Box::leak(format!("Search: {} results", count).into_boxed_str());
                (text, Color::from_rgb(100, 200, 100))
            },
            SearchStatus::NoResults => ("Search: No results", Color::from_rgb(255, 193, 7)),
            SearchStatus::Error(ref msg) => {
                let text: &'static str = Box::leak(format!("Search: Error - {}", msg).into_boxed_str());
                (text, Color::from_rgb(220, 53, 69))
            },
        };
        
        self.search_status.set_value(text);
        self.search_status.set_text_color(color);
        self.search_status.redraw();
    }
    
    // 显示进度指示器
    pub fn show_progress(&mut self, progress: f32, message: Option<&str>) {
        // 简单的进度显示
        let progress_text = format!("{:.0}%", progress * 100.0);
        self.progress_frame.set_label(&progress_text);
        
        if let Some(msg) = message {
            self.set_status(msg);
        }
        
        self.progress_frame.redraw();
    }
    
    // 隐藏进度指示器
    pub fn hide_progress(&mut self) {
        self.progress_frame.set_label("");
        self.progress_frame.redraw();
    }
    
    // 显示加载状态
    pub fn set_loading(&mut self, loading: bool, message: Option<&str>) {
        if loading {
            let msg = message.unwrap_or("Loading...");
            self.set_status(msg);
            self.show_progress(0.0, None);
        } else {
            self.hide_progress();
            self.set_status("Ready");
        }
    }
    
    // 显示错误状态
    pub fn show_error(&mut self, error: &str) {
        self.set_status(&format!("Error: {}", error));
        // TODO: 可以添加错误图标或颜色变化
    }
    
    // 显示成功状态
    pub fn show_success(&mut self, message: &str) {
        self.set_temp_status(&format!("✓ {}", message), 3000);
    }
    
    // 清除所有状态
    pub fn reset(&mut self) {
        self.set_status("Ready");
        self.set_file_count(0, None);
        self.set_selected_count(0);
        self.set_search_status(SearchStatus::Ready);
        self.hide_progress();
    }
    
    // 获取容器引用（用于主窗口布局）
    pub fn widget(&mut self) -> &mut Group {
        &mut self.container
    }
    
    // 更新整体统计信息
    pub fn update_stats(&mut self, stats: &AppStats) {
        self.set_file_count(stats.total_files, Some(stats.visible_files));
        self.set_selected_count(stats.selected_files);
        self.set_database_status(stats.db_connected, Some(stats.db_size_mb));
        
        if let Some(ref config_path) = stats.config_path {
            self.set_config_status(Some(config_path));
        } else {
            self.set_config_status(None);
        }
    }
}

// 搜索状态枚举
#[derive(Debug, Clone)]
pub enum SearchStatus {
    Ready,
    Searching,
    Results(usize),
    NoResults,
    Error(String),
}

// 应用统计信息结构
#[derive(Debug, Default)]
pub struct AppStats {
    pub total_files: usize,
    pub visible_files: usize,
    pub selected_files: usize,
    pub db_connected: bool,
    pub db_size_mb: f64,
    pub config_path: Option<PathBuf>,
    pub log_level: String,
}

impl AppStats {
    pub fn new() -> Self {
        Self {
            total_files: 0,
            visible_files: 0,
            selected_files: 0,
            db_connected: false,
            db_size_mb: 0.0,
            config_path: None,
            log_level: "INFO".to_string(),
        }
    }
}