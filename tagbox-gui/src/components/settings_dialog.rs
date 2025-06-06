use fltk::{
    prelude::*,
    window::Window,
    group::{Flex, FlexType, Tabs, Group},
    input::{Input, IntInput},
    button::Button,
    menu::Choice,
    enums::Color,
    frame::Frame,
    text::{TextEditor, TextBuffer},
    output::Output,
    dialog::NativeFileChooser,
};
use std::sync::mpsc::Sender;
use std::path::{Path, PathBuf};
use tagbox_core::config::AppConfig;
use crate::state::AppEvent;
use crate::utils::open_file;

pub struct SettingsDialog {
    window: Window,
    
    // 配置文件管理
    config_path_output: Output,
    browse_config_btn: Button,
    edit_config_btn: Button,
    new_config_btn: Button,
    reload_config_btn: Button,
    
    // 日志设置
    log_level_choice: Choice,
    log_file_output: Output,
    browse_log_btn: Button,
    open_log_btn: Button,
    clear_log_btn: Button,
    
    // 数据库设置
    db_path_output: Output,
    browse_db_btn: Button,
    backup_db_btn: Button,
    rebuild_index_btn: Button,
    
    // 导入设置
    import_path_input: Input,
    browse_import_btn: Button,
    auto_extract_checkbox: fltk::button::CheckButton,
    auto_move_checkbox: fltk::button::CheckButton,
    
    // 界面设置
    theme_choice: Choice,
    font_size_input: IntInput,
    language_choice: Choice,
    
    // 操作按钮
    save_btn: Button,
    cancel_btn: Button,
    reset_btn: Button,
    
    // 状态
    current_config: Option<AppConfig>,
    config_path: Option<PathBuf>,
    event_sender: Sender<AppEvent>,
}

impl SettingsDialog {
    pub fn new(event_sender: Sender<AppEvent>) -> Self {
        let mut window = Window::new(200, 200, 600, 500, "Settings");
        window.make_modal(true);
        window.set_color(Color::from_rgb(248, 249, 250));
        
        let mut tabs = Tabs::new(10, 10, 580, 430, None);
        
        // 配置管理标签页
        let mut config_tab = Group::new(10, 35, 580, 405, "Configuration\t");
        config_tab.set_color(Color::White);
        Self::create_config_tab(&mut config_tab);
        config_tab.end();
        
        // 日志设置标签页
        let mut logging_tab = Group::new(10, 35, 580, 405, "Logging\t");
        logging_tab.set_color(Color::White);
        Self::create_logging_tab(&mut logging_tab);
        logging_tab.end();
        
        // 数据库设置标签页
        let mut database_tab = Group::new(10, 35, 580, 405, "Database\t");
        database_tab.set_color(Color::White);
        Self::create_database_tab(&mut database_tab);
        database_tab.end();
        
        // 导入设置标签页
        let mut import_tab = Group::new(10, 35, 580, 405, "Import\t");
        import_tab.set_color(Color::White);
        Self::create_import_tab(&mut import_tab);
        import_tab.end();
        
        // 界面设置标签页
        let mut ui_tab = Group::new(10, 35, 580, 405, "Interface\t");
        ui_tab.set_color(Color::White);
        Self::create_ui_tab(&mut ui_tab);
        ui_tab.end();
        
        tabs.end();
        
        // 底部按钮
        let mut button_flex = Flex::new(10, 450, 580, 40, None);
        button_flex.set_type(FlexType::Row);
        button_flex.set_spacing(10);
        
        let mut save_btn = Button::new(0, 0, 0, 0, "Save Settings");
        save_btn.set_color(Color::from_rgb(40, 167, 69));
        save_btn.set_label_color(Color::White);
        
        let mut reset_btn = Button::new(0, 0, 0, 0, "Reset to Defaults");
        reset_btn.set_color(Color::from_rgb(255, 193, 7));
        reset_btn.set_label_color(Color::Black);
        
        let mut cancel_btn = Button::new(0, 0, 0, 0, "Cancel");
        cancel_btn.set_color(Color::from_rgb(108, 117, 125));
        cancel_btn.set_label_color(Color::White);
        
        button_flex.end();
        window.end();
        
        // 创建控件实例（简化版）
        let config_path_output = Output::new(0, 0, 0, 0, None);
        let browse_config_btn = Button::new(0, 0, 0, 0, "Browse...");
        let edit_config_btn = Button::new(0, 0, 0, 0, "Edit");
        let new_config_btn = Button::new(0, 0, 0, 0, "New");
        let reload_config_btn = Button::new(0, 0, 0, 0, "Reload");
        
        let log_level_choice = Choice::new(0, 0, 0, 0, None);
        let log_file_output = Output::new(0, 0, 0, 0, None);
        let browse_log_btn = Button::new(0, 0, 0, 0, "Browse...");
        let open_log_btn = Button::new(0, 0, 0, 0, "Open");
        let clear_log_btn = Button::new(0, 0, 0, 0, "Clear");
        
        let db_path_output = Output::new(0, 0, 0, 0, None);
        let browse_db_btn = Button::new(0, 0, 0, 0, "Browse...");
        let backup_db_btn = Button::new(0, 0, 0, 0, "Backup");
        let rebuild_index_btn = Button::new(0, 0, 0, 0, "Rebuild Index");
        
        let import_path_input = Input::new(0, 0, 0, 0, None);
        let browse_import_btn = Button::new(0, 0, 0, 0, "Browse...");
        let auto_extract_checkbox = fltk::button::CheckButton::new(0, 0, 0, 0, "Auto extract metadata");
        let auto_move_checkbox = fltk::button::CheckButton::new(0, 0, 0, 0, "Auto move files");
        
        let theme_choice = Choice::new(0, 0, 0, 0, None);
        let font_size_input = IntInput::new(0, 0, 0, 0, None);
        let language_choice = Choice::new(0, 0, 0, 0, None);
        
        Self {
            window,
            config_path_output,
            browse_config_btn,
            edit_config_btn,
            new_config_btn,
            reload_config_btn,
            log_level_choice,
            log_file_output,
            browse_log_btn,
            open_log_btn,
            clear_log_btn,
            db_path_output,
            browse_db_btn,
            backup_db_btn,
            rebuild_index_btn,
            import_path_input,
            browse_import_btn,
            auto_extract_checkbox,
            auto_move_checkbox,
            theme_choice,
            font_size_input,
            language_choice,
            save_btn,
            cancel_btn,
            reset_btn,
            current_config: None,
            config_path: None,
            event_sender,
        }
    }
    
    fn create_config_tab(group: &mut Group) {
        let padding = 20;
        let mut y = 60;
        let label_height = 20;
        let field_height = 30;
        let spacing = 15;
        
        // 当前配置文件
        let _config_label = Frame::new(padding, y, 200, label_height, "Current configuration file:");
        y += label_height + 5;
        
        let _config_path = Output::new(padding, y, 400, field_height, None);
        let _browse_btn = Button::new(padding + 410, y, 80, field_height, "Browse...");
        let _edit_btn = Button::new(padding + 500, y, 60, field_height, "Edit");
        y += field_height + spacing;
        
        // 配置操作
        let _new_btn = Button::new(padding, y, 120, field_height, "New Config");
        let _reload_btn = Button::new(padding + 130, y, 120, field_height, "Reload Config");
        y += field_height + spacing;
        
        // 配置验证状态
        let _status_label = Frame::new(padding, y, 540, label_height, "Configuration Status: Valid");
    }
    
    fn create_logging_tab(group: &mut Group) {
        let padding = 20;
        let mut y = 60;
        let label_height = 20;
        let field_height = 30;
        let spacing = 15;
        
        // 日志级别
        let _level_label = Frame::new(padding, y, 150, label_height, "Log Level:");
        y += label_height + 5;
        
        let mut _level_choice = Choice::new(padding, y, 200, field_height, None);
        y += field_height + spacing;
        
        // 日志文件路径
        let _log_file_label = Frame::new(padding, y, 150, label_height, "Log File:");
        y += label_height + 5;
        
        let _log_path = Output::new(padding, y, 350, field_height, None);
        let _browse_log_btn = Button::new(padding + 360, y, 80, field_height, "Browse...");
        let _open_log_btn = Button::new(padding + 450, y, 60, field_height, "Open");
        y += field_height + spacing;
        
        // 日志操作
        let _clear_log_btn = Button::new(padding, y, 120, field_height, "Clear Log");
        let _export_log_btn = Button::new(padding + 130, y, 120, field_height, "Export Log");
    }
    
    fn create_database_tab(group: &mut Group) {
        let padding = 20;
        let mut y = 60;
        let label_height = 20;
        let field_height = 30;
        let spacing = 15;
        
        // 数据库路径
        let _db_label = Frame::new(padding, y, 150, label_height, "Database File:");
        y += label_height + 5;
        
        let _db_path = Output::new(padding, y, 350, field_height, None);
        let _browse_db_btn = Button::new(padding + 360, y, 80, field_height, "Browse...");
        y += field_height + spacing;
        
        // 数据库操作
        let _backup_btn = Button::new(padding, y, 120, field_height, "Backup DB");
        let _restore_btn = Button::new(padding + 130, y, 120, field_height, "Restore DB");
        let _rebuild_btn = Button::new(padding + 260, y, 120, field_height, "Rebuild Index");
        y += field_height + spacing;
        
        // 数据库统计
        let _stats_label = Frame::new(padding, y, 540, label_height, "Database Statistics:");
        y += label_height + 10;
        let _files_count = Frame::new(padding + 20, y, 250, label_height, "Total Files: 0");
        let _size_info = Frame::new(padding + 280, y, 250, label_height, "Database Size: 0 MB");
    }
    
    fn create_import_tab(group: &mut Group) {
        let padding = 20;
        let mut y = 60;
        let label_height = 20;
        let field_height = 30;
        let spacing = 15;
        
        // 默认导入路径
        let _import_label = Frame::new(padding, y, 150, label_height, "Default Import Path:");
        y += label_height + 5;
        
        let _import_path = Input::new(padding, y, 350, field_height, None);
        let _browse_import_btn = Button::new(padding + 360, y, 80, field_height, "Browse...");
        y += field_height + spacing;
        
        // 导入选项
        let _auto_extract = fltk::button::CheckButton::new(padding, y, 250, field_height, "Auto extract metadata");
        y += field_height + 10;
        let _auto_move = fltk::button::CheckButton::new(padding, y, 250, field_height, "Auto move imported files");
        y += field_height + 10;
        let _auto_categorize = fltk::button::CheckButton::new(padding, y, 250, field_height, "Auto categorize by file type");
    }
    
    fn create_ui_tab(group: &mut Group) {
        let padding = 20;
        let mut y = 60;
        let label_height = 20;
        let field_height = 30;
        let spacing = 15;
        
        // 主题选择
        let _theme_label = Frame::new(padding, y, 150, label_height, "Theme:");
        y += label_height + 5;
        
        let mut _theme_choice = Choice::new(padding, y, 200, field_height, None);
        y += field_height + spacing;
        
        // 字体大小
        let _font_label = Frame::new(padding, y, 150, label_height, "Font Size:");
        y += label_height + 5;
        
        let _font_size = IntInput::new(padding, y, 100, field_height, None);
        y += field_height + spacing;
        
        // 语言选择
        let _language_label = Frame::new(padding, y, 150, label_height, "Language:");
        y += label_height + 5;
        
        let mut _language_choice = Choice::new(padding, y, 200, field_height, None);
    }
    
    pub fn show(&mut self) {
        self.window.show();
    }
    
    pub fn hide(&mut self) {
        self.window.hide();
    }

    pub fn shown(&self) -> bool {
        self.window.shown()
    }
    
    pub fn load_config(&mut self, config: AppConfig, config_path: Option<PathBuf>) {
        self.current_config = Some(config.clone());
        self.config_path = config_path.clone();
        
        // 更新界面显示
        if let Some(path) = &config_path {
            self.config_path_output.set_value(&path.to_string_lossy());
        }
        
        // 设置日志级别
        self.update_log_level_from_config(&config);
        
        // 更新其他配置项
        self.populate_ui_settings(&config);
    }
    
    fn update_log_level_from_config(&mut self, config: &AppConfig) {
        // TODO: 从配置中读取日志级别并设置到选择框
        self.log_level_choice.clear();
        self.log_level_choice.add_choice("ERROR");
        self.log_level_choice.add_choice("WARN");
        self.log_level_choice.add_choice("INFO");
        self.log_level_choice.add_choice("DEBUG");
        self.log_level_choice.add_choice("TRACE");
        self.log_level_choice.set_value(2); // 默认 INFO
    }
    
    fn populate_ui_settings(&mut self, config: &AppConfig) {
        // 主题设置
        self.theme_choice.clear();
        self.theme_choice.add_choice("Default");
        self.theme_choice.add_choice("Dark");
        self.theme_choice.add_choice("Light");
        self.theme_choice.set_value(0);
        
        // 语言设置
        self.language_choice.clear();
        self.language_choice.add_choice("English");
        self.language_choice.add_choice("中文");
        self.language_choice.set_value(0);
        
        // 字体大小
        self.font_size_input.set_value("12");
    }
    
    pub fn set_callbacks(&mut self) {
        // 编辑配置文件按钮
        let config_path = self.config_path.clone();
        self.edit_config_btn.set_callback(move |_| {
            if let Some(path) = &config_path {
                if let Err(e) = Self::edit_config_file(path) {
                    eprintln!("Failed to open config file: {}", e);
                }
            }
        });
        
        // 新建配置文件按钮
        let sender = self.event_sender.clone();
        self.new_config_btn.set_callback(move |_| {
            if let Err(e) = Self::create_new_config() {
                eprintln!("Failed to create new config: {}", e);
            } else {
                let _ = sender.send(AppEvent::RefreshView);
            }
        });
        
        // 浏览配置文件按钮
        let sender = self.event_sender.clone();
        self.browse_config_btn.set_callback(move |_| {
            if let Some(path) = Self::browse_config_file() {
                // TODO: 加载选中的配置文件
                println!("Selected config file: {}", path.display());
            }
        });
        
        // 保存设置按钮
        let sender = self.event_sender.clone();
        self.save_btn.set_callback(move |_| {
            let _ = sender.send(AppEvent::RefreshView);
        });
        
        // 取消按钮
        let sender = self.event_sender.clone();
        self.cancel_btn.set_callback(move |_| {
            // 关闭对话框
        });
        
        // 打开日志文件按钮
        self.open_log_btn.set_callback(move |_| {
            // TODO: 打开日志文件
        });
        
        // 清除日志按钮
        self.clear_log_btn.set_callback(move |_| {
            if Self::confirm_clear_log() {
                if let Err(e) = Self::clear_log_file() {
                    eprintln!("Failed to clear log: {}", e);
                }
            }
        });
    }
    
    fn edit_config_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        open_file(path)
    }
    
    fn create_new_config() -> Result<(), Box<dyn std::error::Error>> {
        let mut dialog = NativeFileChooser::new(fltk::dialog::NativeFileChooserType::BrowseSaveFile);
        dialog.set_title("Create New Configuration File");
        dialog.set_filter("TOML Files\t*.toml");
        dialog.show();
        
        let path = dialog.filename();
        if !path.to_string_lossy().is_empty() {
            // TODO: 创建默认配置文件
            std::fs::write(&path, Self::default_config_content())?;
            open_file(&path)?;
        }
        
        Ok(())
    }
    
    fn browse_config_file() -> Option<PathBuf> {
        let mut dialog = NativeFileChooser::new(fltk::dialog::NativeFileChooserType::BrowseFile);
        dialog.set_title("Select Configuration File");
        dialog.set_filter("TOML Files\t*.toml");
        dialog.show();
        
        let path = dialog.filename();
        if !path.to_string_lossy().is_empty() {
            Some(path)
        } else {
            None
        }
    }
    
    fn default_config_content() -> &'static str {
        r#"# TagBox Configuration File
# This file configures the TagBox file management system

[database]
path = "tagbox.db"
backup_on_startup = false

[import]
default_category = "Uncategorized"
auto_extract_metadata = true
auto_move_files = false

[search]
max_results = 100
enable_full_text_search = true

[logging]
level = "INFO"
file = "tagbox.log"
max_size_mb = 10
max_files = 5

[ui]
theme = "default"
font_size = 12
language = "en"
"#
    }
    
    fn confirm_clear_log() -> bool {
        let choice = fltk::dialog::choice2_default(
            "Are you sure you want to clear the log file?",
            "Yes",
            "No",
            ""
        );
        choice == Some(0)
    }
    
    fn clear_log_file() -> Result<(), Box<dyn std::error::Error>> {
        // TODO: 实现清除日志文件的逻辑
        Ok(())
    }
    
    pub fn apply_settings(&self) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: 应用设置更改
        Ok(())
    }
    
    pub fn reset_to_defaults(&mut self) {
        // TODO: 重置所有设置到默认值
        self.log_level_choice.set_value(2); // INFO
        self.theme_choice.set_value(0); // Default
        self.language_choice.set_value(0); // English
        self.font_size_input.set_value("12");
        self.auto_extract_checkbox.set_value(true);
        self.auto_move_checkbox.set_value(false);
    }
}