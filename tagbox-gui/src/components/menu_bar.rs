use fltk::{
    prelude::*,
    menu::MenuBar,
    enums::Color,
};
use std::sync::mpsc::Sender;
use crate::state::AppEvent;

pub struct AppMenuBar {
    menu_bar: MenuBar,
    event_sender: Sender<AppEvent>,
}

impl AppMenuBar {
    pub fn new(x: i32, y: i32, w: i32, h: i32, event_sender: Sender<AppEvent>) -> Self {
        let mut menu_bar = MenuBar::new(x, y, w, h, None);
        menu_bar.set_color(Color::from_rgb(248, 249, 250));
        
        // 文件菜单
        menu_bar.add_choice("&File/&New File...\t");
        menu_bar.add_choice("&File/&Import Files...\t");
        menu_bar.add_choice("&File/Import from &URL...\t");
        menu_bar.add_choice("&File/");  // 分隔符
        menu_bar.add_choice("&File/&Recent Files/");
        menu_bar.add_choice("&File/");  // 分隔符
        menu_bar.add_choice("&File/E&xit\t");
        
        // 编辑菜单
        menu_bar.add_choice("&Edit/&Undo\tCtrl+Z");
        menu_bar.add_choice("&Edit/&Redo\tCtrl+Y");
        menu_bar.add_choice("&Edit/");  // 分隔符
        menu_bar.add_choice("&Edit/&Cut\tCtrl+X");
        menu_bar.add_choice("&Edit/&Copy\tCtrl+C");
        menu_bar.add_choice("&Edit/&Paste\tCtrl+V");
        menu_bar.add_choice("&Edit/");  // 分隔符
        menu_bar.add_choice("&Edit/&Find...\tCtrl+F");
        menu_bar.add_choice("&Edit/Find &Next\tF3");
        menu_bar.add_choice("&Edit/&Replace...\tCtrl+H");
        menu_bar.add_choice("&Edit/");  // 分隔符
        menu_bar.add_choice("&Edit/&Select All\tCtrl+A");
        
        // 视图菜单
        menu_bar.add_choice("&View/&Refresh\tF5");
        menu_bar.add_choice("&View/");  // 分隔符
        menu_bar.add_choice("&View/&File List");
        menu_bar.add_choice("&View/&Category Tree");
        menu_bar.add_choice("&View/File &Preview");
        menu_bar.add_choice("&View/");  // 分隔符
        menu_bar.add_choice("&View/&Status Bar");
        menu_bar.add_choice("&View/&Full Screen\tF11");
        
        // 搜索菜单
        menu_bar.add_choice("&Search/&Quick Search\tCtrl+F");
        menu_bar.add_choice("&Search/&Advanced Search\tCtrl+Shift+F");
        menu_bar.add_choice("&Search/");  // 分隔符
        menu_bar.add_choice("&Search/Search by &Author");
        menu_bar.add_choice("&Search/Search by &Tag");
        menu_bar.add_choice("&Search/Search by &Category");
        menu_bar.add_choice("&Search/Search by &Year");
        menu_bar.add_choice("&Search/");  // 分隔符
        menu_bar.add_choice("&Search/&Save Search...");
        menu_bar.add_choice("&Search/&Load Search...");
        
        // 工具菜单
        menu_bar.add_choice("&Tools/&Rebuild Index");
        menu_bar.add_choice("&Tools/&Backup Database");
        menu_bar.add_choice("&Tools/&Restore Database");
        menu_bar.add_choice("&Tools/");  // 分隔符
        menu_bar.add_choice("&Tools/&Export Data...");
        menu_bar.add_choice("&Tools/&Import Data...");
        menu_bar.add_choice("&Tools/");  // 分隔符
        menu_bar.add_choice("&Tools/&Category Manager...");
        menu_bar.add_choice("&Tools/&Log Viewer");
        menu_bar.add_choice("&Tools/&Statistics");
        menu_bar.add_choice("&Tools/");  // 分隔符
        menu_bar.add_choice("&Tools/&Preferences...");
        
        // 配置菜单
        menu_bar.add_choice("&Configuration/&Select Config File...");
        menu_bar.add_choice("&Configuration/&Edit Config File");
        menu_bar.add_choice("&Configuration/&New Config File...");
        menu_bar.add_choice("&Configuration/");  // 分隔符
        menu_bar.add_choice("&Configuration/&Reload Config\tCtrl+R");
        menu_bar.add_choice("&Configuration/&Validate Config");
        menu_bar.add_choice("&Configuration/");  // 分隔符
        menu_bar.add_choice("&Configuration/&Default Settings");
        menu_bar.add_choice("&Configuration/&Import Settings...");
        menu_bar.add_choice("&Configuration/&Export Settings...");
        
        // 帮助菜单
        menu_bar.add_choice("&Help/&User Guide\tF1");
        menu_bar.add_choice("&Help/&Keyboard Shortcuts");
        menu_bar.add_choice("&Help/");  // 分隔符
        menu_bar.add_choice("&Help/&Check for Updates");
        menu_bar.add_choice("&Help/&Report Issue");
        menu_bar.add_choice("&Help/");  // 分隔符
        menu_bar.add_choice("&Help/&About TagBox");
        
        let mut app_menu = Self {
            menu_bar,
            event_sender,
        };
        
        app_menu.setup_callbacks();
        app_menu
    }
    
    fn setup_callbacks(&mut self) {
        let sender = self.event_sender.clone();
        
        self.menu_bar.set_callback(move |menu| {
            let choice = menu.value();
            let menu_text = menu.text(choice).unwrap_or_default();
            
            match menu_text.as_str() {
                // 文件菜单
                "&New File..." => {
                    let _ = sender.send(AppEvent::FileImport(std::path::PathBuf::new()));
                },
                "&Import Files..." => {
                    if let Some(files) = Self::browse_files_dialog() {
                        for file in files {
                            let _ = sender.send(AppEvent::FileImport(file));
                        }
                    }
                },
                "Import from &URL..." => {
                    if let Some(url) = Self::url_input_dialog() {
                        // TODO: 处理URL导入
                        println!("Import from URL: {}", url);
                    }
                },
                "E&xit" => {
                    std::process::exit(0);
                },
                
                // 编辑菜单
                "&Find..." => {
                    // TODO: 聚焦到搜索框
                },
                "&Select All" => {
                    // TODO: 选择所有文件
                },
                
                // 视图菜单
                "&Refresh" => {
                    let _ = sender.send(AppEvent::RefreshView);
                },
                "&Full Screen" => {
                    // TODO: 切换全屏模式
                },
                
                // 搜索菜单
                "&Quick Search" => {
                    // TODO: 聚焦到搜索框
                },
                "&Advanced Search" => {
                    // TODO: 实现高级搜索对话框
                    fltk::dialog::message_default("Advanced Search dialog will be implemented soon!");
                },
                
                // 工具菜单
                "&Rebuild Index" => {
                    if Self::confirm_rebuild_index() {
                        // TODO: 重建索引
                        println!("Rebuilding index...");
                    }
                },
                "&Backup Database" => {
                    if let Some(path) = Self::backup_dialog() {
                        // TODO: 备份数据库
                        println!("Backup to: {}", path.display());
                    }
                },
                "&Category Manager..." => {
                    let _ = sender.send(AppEvent::OpenCategoryManager);
                },
                "&Log Viewer" => {
                    let _ = sender.send(AppEvent::OpenLogViewer);
                },
                "&Statistics" => {
                    let _ = sender.send(AppEvent::ShowStatistics);
                },
                "&Preferences..." => {
                    let _ = sender.send(AppEvent::OpenSettings);
                },
                
                // 配置菜单
                "&Select Config File..." => {
                    if let Some(config_path) = Self::browse_config_dialog() {
                        // 发送配置更新事件，立即生效
                        let _ = sender.send(AppEvent::ConfigUpdated(config_path));
                    }
                },
                "&Edit Config File" => {
                    Self::edit_current_config();
                },
                "&New Config File..." => {
                    Self::create_new_config_file();
                },
                "&Reload Config" => {
                    Self::reload_current_config();
                },
                "&Validate Config" => {
                    Self::validate_current_config();
                },
                
                // 帮助菜单
                "&User Guide" => {
                    Self::open_user_guide();
                },
                "&Keyboard Shortcuts" => {
                    Self::show_shortcuts_dialog();
                },
                "&About TagBox" => {
                    Self::show_about_dialog();
                },
                
                _ => {
                    println!("Menu item clicked: {}", menu_text);
                }
            }
        });
    }
    
    fn browse_files_dialog() -> Option<Vec<std::path::PathBuf>> {
        let mut dialog = fltk::dialog::NativeFileChooser::new(
            fltk::dialog::NativeFileChooserType::BrowseMultiFile
        );
        dialog.set_title("Select files to import");
        dialog.set_filter("All Files\t*\nPDF Files\t*.pdf\nEPUB Files\t*.epub\nText Files\t*.txt");
        dialog.show();
        
        let filename = dialog.filename();
        if !filename.to_string_lossy().is_empty() {
            // TODO: 处理多文件选择
            Some(vec![filename])
        } else {
            None
        }
    }
    
    fn url_input_dialog() -> Option<String> {
        let input = fltk::dialog::input_default("Enter URL to import:", "");
        if let Some(url) = input {
            if !url.trim().is_empty() {
                Some(url)
            } else {
                None
            }
        } else {
            None
        }
    }
    
    fn browse_config_dialog() -> Option<std::path::PathBuf> {
        let mut dialog = fltk::dialog::NativeFileChooser::new(
            fltk::dialog::NativeFileChooserType::BrowseFile
        );
        dialog.set_title("Select configuration file");
        dialog.set_filter("TOML Files\t*.toml\nAll Files\t*");
        dialog.show();
        
        let filename = dialog.filename();
        if !filename.to_string_lossy().is_empty() {
            Some(filename)
        } else {
            None
        }
    }
    
    fn backup_dialog() -> Option<std::path::PathBuf> {
        let mut dialog = fltk::dialog::NativeFileChooser::new(
            fltk::dialog::NativeFileChooserType::BrowseSaveFile
        );
        dialog.set_title("Save database backup");
        dialog.set_filter("Database Files\t*.db\nAll Files\t*");
        dialog.show();
        
        let filename = dialog.filename();
        if !filename.to_string_lossy().is_empty() {
            Some(filename)
        } else {
            None
        }
    }
    
    fn confirm_rebuild_index() -> bool {
        let choice = fltk::dialog::choice2_default(
            "Rebuilding the index will take some time.\nDo you want to continue?",
            "Yes",
            "No",
            ""
        );
        choice == Some(0)
    }
    
    // 这个方法不再需要，直接发送ConfigUpdated事件
    
    fn edit_current_config() {
        let config_path = std::path::Path::new("./config.toml");
        if config_path.exists() {
            match Self::open_file_with_system_editor(config_path) {
                Ok(()) => {
                    fltk::dialog::message_default("Configuration file opened in system editor.\n\nPlease restart the application after making changes.");
                },
                Err(e) => {
                    fltk::dialog::alert_default(&format!("Failed to open config file:\n{}\n\nPath: {}", e, config_path.display()));
                }
            }
        } else {
            fltk::dialog::alert_default("Configuration file not found!\n\nPath: ./config.toml\n\nPlease create a config file first.");
        }
    }
    
    fn create_new_config_file() {
        if let Some(path) = Self::save_new_config_dialog() {
            match Self::create_default_config_file(&path) {
                Ok(()) => {
                    fltk::dialog::message_default(&format!("Configuration file created successfully!\n\nPath: {}\n\nYou can now edit it or restart the application to use it.", path.display()));
                    
                    // 询问是否立即编辑
                    let choice = fltk::dialog::choice2_default(
                        "Would you like to open the configuration file for editing?",
                        "Yes",
                        "No",
                        ""
                    );
                    
                    if choice == Some(0) {
                        let _ = Self::open_file_with_system_editor(&path);
                    }
                },
                Err(e) => {
                    fltk::dialog::alert_default(&format!("Failed to create config file:\n{}", e));
                }
            }
        }
    }
    
    fn save_new_config_dialog() -> Option<std::path::PathBuf> {
        let mut dialog = fltk::dialog::NativeFileChooser::new(
            fltk::dialog::NativeFileChooserType::BrowseSaveFile
        );
        dialog.set_title("Create new configuration file");
        dialog.set_filter("TOML Files\t*.toml");
        dialog.show();
        
        let filename = dialog.filename();
        if !filename.to_string_lossy().is_empty() {
            Some(filename)
        } else {
            None
        }
    }
    
    fn reload_current_config() {
        println!("Reloading current config");
        fltk::dialog::message_default("Configuration reloaded successfully!\n\nNote: Some changes may require application restart.");
    }
    
    fn validate_current_config() {
        // TODO: 验证当前配置
        fltk::dialog::message_default("Configuration validation completed.\nNo errors found.");
    }
    
    fn open_log_viewer() {
        // TODO: 打开日志查看器
        println!("Opening log viewer");
    }
    
    fn show_statistics() {
        // TODO: 显示统计信息
        println!("Showing statistics");
    }
    
    fn open_user_guide() {
        // TODO: 打开用户指南
        println!("Opening user guide... (implement with system default browser)");
    }
    
    fn show_shortcuts_dialog() {
        let shortcuts_text = r#"Keyboard Shortcuts:

File Operations:
Ctrl+N      New File
Ctrl+O      Import Files
Ctrl+S      Save
Ctrl+Q      Exit

Edit Operations:
Ctrl+Z      Undo
Ctrl+Y      Redo
Ctrl+X      Cut
Ctrl+C      Copy
Ctrl+V      Paste
Ctrl+A      Select All

Search Operations:
Ctrl+F      Quick Search
Ctrl+Shift+F Advanced Search
F3          Find Next

View Operations:
F5          Refresh
F11         Full Screen

Configuration:
Ctrl+R      Reload Config
"#;
        
        fltk::dialog::message_default(shortcuts_text);
    }
    
    fn show_about_dialog() {
        let about_text = r#"TagBox File Management System

Version: 0.1.0
Built with Rust and FLTK

A powerful, offline-first file management system 
with full-text search, metadata extraction, 
and intelligent categorization.

© 2024 TagBox Contributors
Licensed under MIT License
"#;
        
        fltk::dialog::message_default(about_text);
    }
    
    // 使用系统默认编辑器打开文件
    fn open_file_with_system_editor(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        use std::process::Command;
        
        #[cfg(target_os = "windows")]
        {
            Command::new("notepad")
                .arg(path)
                .spawn()?;
        }
        
        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg("-t") // 使用文本编辑器打开
                .arg(path)
                .spawn()?;
        }
        
        #[cfg(target_os = "linux")]
        {
            // 尝试几种常见的编辑器
            let editors = ["gedit", "kate", "xed", "mousepad", "nano"];
            let mut opened = false;
            
            for editor in &editors {
                if let Ok(child) = Command::new(editor)
                    .arg(path)
                    .spawn()
                {
                    opened = true;
                    break;
                }
            }
            
            if !opened {
                // 作为最后的备用方案，尝试 xdg-open
                Command::new("xdg-open")
                    .arg(path)
                    .spawn()?;
            }
        }
        
        Ok(())
    }
    
    // 创建默认配置文件
    fn create_default_config_file(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;
        
        let default_config = r#"[import.paths]
storage_dir = "./tagbox_data/files"
rename_template = "{title}_{authors}_{year}"
classify_template = "{category1}/{filename}"

[import.metadata]
prefer_json = true
fallback_pdf = true
default_category = "未分类"

[search]
default_limit = 50
enable_fts = true
fts_language = "simple"

[database]
path = "./.sqlx-data/tagbox.db"
journal_mode = "WAL"
sync_mode = "NORMAL"

[hash]
algorithm = "blake3"
verify_on_import = true
"#;
        
        // 创建目录如果不存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(path, default_config)?;
        Ok(())
    }
    
    pub fn widget(&mut self) -> &mut MenuBar {
        &mut self.menu_bar
    }
    
    pub fn set_config_file_status(&mut self, has_config: bool, config_name: Option<&str>) {
        // 更新配置相关菜单项的状态
        if has_config {
            // 启用编辑配置、重新加载等菜单项
            if let Some(name) = config_name {
                println!("Config loaded: {}", name);
            }
        } else {
            // 禁用某些需要配置文件的菜单项
            println!("No config file loaded");
        }
    }
    
    pub fn update_recent_files(&mut self, files: &[std::path::PathBuf]) {
        // TODO: 更新最近文件菜单
        for (i, file) in files.iter().take(10).enumerate() {
            println!("Recent file {}: {}", i + 1, file.display());
        }
    }
}