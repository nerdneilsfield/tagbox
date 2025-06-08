use fltk::{
    prelude::*,
    window::Window,
    group::{Flex, FlexType, Tabs, Group},
    input::{Input, IntInput},
    button::{Button, CheckButton},
    enums::{Color, Align},
    frame::Frame,
    output::Output,
    dialog::{NativeFileChooser, FileDialogType},
};
use std::sync::mpsc::Sender;
use std::path::{Path, PathBuf};
use std::fs;
use tagbox_core::config::AppConfig;
use crate::state::AppEvent;
use chrono;

pub struct SettingsDialog {
    window: Window,
    
    // 配置文件管理
    config_path_output: Output,
    browse_config_btn: Button,
    edit_config_btn: Button,
    reload_config_btn: Button,
    
    // 数据库设置
    db_path_output: Output,
    browse_db_btn: Button,
    backup_db_btn: Button,
    rebuild_index_btn: Button,
    
    // 导入设置
    import_path_input: Input,
    browse_import_btn: Button,
    auto_extract_checkbox: CheckButton,
    auto_move_checkbox: CheckButton,
    
    // 搜索设置
    enable_fts_checkbox: CheckButton,
    max_results_input: IntInput,
    
    // 操作按钮
    save_btn: Button,
    cancel_btn: Button,
    reset_btn: Button,
    
    // 状态
    current_config: Option<AppConfig>,
    config_path: Option<PathBuf>,
    modified: bool,
    event_sender: Sender<AppEvent>,
}

impl SettingsDialog {
    pub fn new(event_sender: Sender<AppEvent>) -> Self {
        let mut window = Window::new(200, 200, 700, 550, "TagBox Settings");
        window.make_modal(true);
        window.set_color(Color::from_rgb(248, 249, 250));
        
        let tabs = Tabs::new(10, 10, 680, 480, None);
        
        // 配置文件标签页
        let mut config_tab = Group::new(10, 35, 680, 455, "Configuration\t");
        config_tab.set_color(Color::White);
        
        // 数据库设置标签页
        let mut database_tab = Group::new(10, 35, 680, 455, "Database\t");
        database_tab.set_color(Color::White);
        
        // 导入设置标签页
        let mut import_tab = Group::new(10, 35, 680, 455, "Import\t");
        import_tab.set_color(Color::White);
        
        // 搜索设置标签页
        let mut search_tab = Group::new(10, 35, 680, 455, "Search\t");
        search_tab.set_color(Color::White);
        
        // 在各个标签页中创建控件
        let (config_path_output, browse_config_btn, edit_config_btn, reload_config_btn) = 
            Self::create_config_tab(&mut config_tab);
        config_tab.end();
        
        let (db_path_output, browse_db_btn, backup_db_btn, rebuild_index_btn) = 
            Self::create_database_tab(&mut database_tab);
        database_tab.end();
        
        let (import_path_input, browse_import_btn, auto_extract_checkbox, auto_move_checkbox) = 
            Self::create_import_tab(&mut import_tab);
        import_tab.end();
        
        let (enable_fts_checkbox, max_results_input) = 
            Self::create_search_tab(&mut search_tab);
        search_tab.end();
        
        tabs.end();
        
        // 底部按钮
        let mut button_flex = Flex::new(10, 500, 680, 40, None);
        button_flex.set_type(FlexType::Row);
        button_flex.set_spacing(10);
        
        // 空间填充
        let spacer = Frame::new(0, 0, 0, 0, None);
        button_flex.fixed(&spacer, 300);
        
        let mut save_btn = Button::new(0, 0, 100, 30, "Save");
        save_btn.set_color(Color::from_rgb(40, 167, 69));
        save_btn.set_label_color(Color::White);
        button_flex.fixed(&save_btn, 100);
        
        let mut reset_btn = Button::new(0, 0, 100, 30, "Reset");
        reset_btn.set_color(Color::from_rgb(255, 193, 7));
        reset_btn.set_label_color(Color::Black);
        button_flex.fixed(&reset_btn, 100);
        
        let mut cancel_btn = Button::new(0, 0, 100, 30, "Cancel");
        cancel_btn.set_color(Color::from_rgb(108, 117, 125));
        cancel_btn.set_label_color(Color::White);
        button_flex.fixed(&cancel_btn, 100);
        
        button_flex.end();
        window.end();
        
        let mut dialog = Self {
            window,
            config_path_output,
            browse_config_btn,
            edit_config_btn,
            reload_config_btn,
            db_path_output,
            browse_db_btn,
            backup_db_btn,
            rebuild_index_btn,
            import_path_input,
            browse_import_btn,
            auto_extract_checkbox,
            auto_move_checkbox,
            enable_fts_checkbox,
            max_results_input,
            save_btn,
            cancel_btn,
            reset_btn,
            current_config: None,
            config_path: None,
            modified: false,
            event_sender,
        };
        
        dialog.setup_callbacks();
        dialog
    }
    
    fn create_config_tab(group: &mut Group) -> (Output, Button, Button, Button) {
        let mut flex = Flex::new(20, 50, 640, 400, None);
        flex.set_type(FlexType::Column);
        flex.set_spacing(20);
        
        // 配置文件路径组
        let config_group = Group::new(0, 0, 640, 80, None);
        let mut config_label = Frame::new(0, 0, 200, 25, "当前配置文件:");
        config_label.set_align(Align::Left | Align::Inside);
        
        let mut path_flex = Flex::new(0, 30, 640, 40, None);
        path_flex.set_type(FlexType::Row);
        path_flex.set_spacing(10);
        
        let mut config_path_output = Output::new(0, 0, 0, 30, None);
        config_path_output.set_color(Color::from_rgb(248, 249, 250));
        path_flex.fixed(&config_path_output, 400);
        
        let mut browse_config_btn = Button::new(0, 0, 80, 30, "Browse...");
        browse_config_btn.set_color(Color::from_rgb(108, 117, 125));
        browse_config_btn.set_label_color(Color::White);
        path_flex.fixed(&browse_config_btn, 80);
        
        let mut edit_config_btn = Button::new(0, 0, 60, 30, "Edit");
        edit_config_btn.set_color(Color::from_rgb(23, 162, 184));
        edit_config_btn.set_label_color(Color::White);
        path_flex.fixed(&edit_config_btn, 60);
        
        let mut reload_config_btn = Button::new(0, 0, 80, 30, "Reload");
        reload_config_btn.set_color(Color::from_rgb(40, 167, 69));
        reload_config_btn.set_label_color(Color::White);
        path_flex.fixed(&reload_config_btn, 80);
        
        path_flex.end();
        config_group.end();
        flex.fixed(&config_group, 80);
        
        // 配置说明
        let mut desc_frame = Frame::new(0, 0, 640, 120, 
            "配置文件包含 TagBox 的所有设置，包括数据库路径、导入目录等。\n您可以直接编辑配置文件，或者使用下面的设置选项。\n\n支持的配置选项：\n• 数据库位置和连接设置\n• 文件导入和存储路径\n• 搜索和索引配置");
        desc_frame.set_align(Align::Left | Align::Top | Align::Inside);
        desc_frame.set_label_color(Color::from_rgb(108, 117, 125));
        flex.fixed(&desc_frame, 120);
        
        flex.end();
        group.add(&flex);
        
        (config_path_output, browse_config_btn, edit_config_btn, reload_config_btn)
    }
    
    fn create_database_tab(group: &mut Group) -> (Output, Button, Button, Button) {
        let mut flex = Flex::new(20, 50, 640, 400, None);
        flex.set_type(FlexType::Column);
        flex.set_spacing(20);
        
        // 数据库路径组
        let db_group = Group::new(0, 0, 640, 80, None);
        let mut db_label = Frame::new(0, 0, 200, 25, "数据库文件路径:");
        db_label.set_align(Align::Left | Align::Inside);
        
        let mut db_flex = Flex::new(0, 30, 640, 40, None);
        db_flex.set_type(FlexType::Row);
        db_flex.set_spacing(10);
        
        let mut db_path_output = Output::new(0, 0, 0, 30, None);
        db_path_output.set_color(Color::from_rgb(248, 249, 250));
        db_flex.fixed(&db_path_output, 500);
        
        let mut browse_db_btn = Button::new(0, 0, 80, 30, "Browse...");
        browse_db_btn.set_color(Color::from_rgb(108, 117, 125));
        browse_db_btn.set_label_color(Color::White);
        db_flex.fixed(&browse_db_btn, 80);
        
        db_flex.end();
        db_group.end();
        flex.fixed(&db_group, 80);
        
        // 数据库操作组
        let ops_group = Group::new(0, 0, 640, 60, None);
        let mut ops_label = Frame::new(0, 0, 200, 25, "数据库操作:");
        ops_label.set_align(Align::Left | Align::Inside);
        
        let mut ops_flex = Flex::new(0, 30, 640, 30, None);
        ops_flex.set_type(FlexType::Row);
        ops_flex.set_spacing(10);
        
        let mut backup_db_btn = Button::new(0, 0, 120, 30, "备份数据库");
        backup_db_btn.set_color(Color::from_rgb(40, 167, 69));
        backup_db_btn.set_label_color(Color::White);
        ops_flex.fixed(&backup_db_btn, 120);
        
        let mut rebuild_index_btn = Button::new(0, 0, 120, 30, "重建索引");
        rebuild_index_btn.set_color(Color::from_rgb(255, 193, 7));
        rebuild_index_btn.set_label_color(Color::Black);
        ops_flex.fixed(&rebuild_index_btn, 120);
        
        // 填充空间
        let spacer = Frame::new(0, 0, 0, 0, None);
        ops_flex.fixed(&spacer, 400);
        
        ops_flex.end();
        ops_group.end();
        flex.fixed(&ops_group, 60);
        
        // 数据库信息
        let mut info_frame = Frame::new(0, 0, 640, 100,
            "数据库设置:\n\n• SQLite 数据库用于存储文件元数据和全文索引\n• 备份功能可以保护您的数据\n• 重建索引可以修复搜索问题");
        info_frame.set_align(Align::Left | Align::Top | Align::Inside);
        info_frame.set_label_color(Color::from_rgb(108, 117, 125));
        flex.fixed(&info_frame, 100);
        
        flex.end();
        group.add(&flex);
        
        (db_path_output, browse_db_btn, backup_db_btn, rebuild_index_btn)
    }
    
    fn create_import_tab(group: &mut Group) -> (Input, Button, CheckButton, CheckButton) {
        let mut flex = Flex::new(20, 50, 640, 400, None);
        flex.set_type(FlexType::Column);
        flex.set_spacing(20);
        
        // 导入路径组
        let import_group = Group::new(0, 0, 640, 80, None);
        let mut import_label = Frame::new(0, 0, 200, 25, "默认导入目录:");
        import_label.set_align(Align::Left | Align::Inside);
        
        let mut import_flex = Flex::new(0, 30, 640, 40, None);
        import_flex.set_type(FlexType::Row);
        import_flex.set_spacing(10);
        
        let mut import_path_input = Input::new(0, 0, 0, 30, None);
        import_path_input.set_color(Color::White);
        import_flex.fixed(&import_path_input, 500);
        
        let mut browse_import_btn = Button::new(0, 0, 80, 30, "Browse...");
        browse_import_btn.set_color(Color::from_rgb(108, 117, 125));
        browse_import_btn.set_label_color(Color::White);
        import_flex.fixed(&browse_import_btn, 80);
        
        import_flex.end();
        import_group.end();
        flex.fixed(&import_group, 80);
        
        // 导入选项组
        let options_group = Group::new(0, 0, 640, 100, None);
        let mut options_label = Frame::new(0, 0, 200, 25, "导入选项:");
        options_label.set_align(Align::Left | Align::Inside);
        
        let auto_extract_checkbox = CheckButton::new(0, 35, 300, 25, "自动提取文件元数据");
        auto_extract_checkbox.set_checked(true);
        
        let auto_move_checkbox = CheckButton::new(0, 65, 300, 25, "导入后移动文件到存储目录");
        auto_move_checkbox.set_checked(false);
        
        options_group.end();
        flex.fixed(&options_group, 100);
        
        // 导入说明
        let mut info_frame = Frame::new(0, 0, 640, 100,
            "文件导入设置:\n\n• 支持 PDF、EPUB、TXT、DOC 等文档格式\n• 自动提取可以获取文件标题、作者等信息\n• 移动文件可以统一管理文档存储");
        info_frame.set_align(Align::Left | Align::Top | Align::Inside);
        info_frame.set_label_color(Color::from_rgb(108, 117, 125));
        flex.fixed(&info_frame, 100);
        
        flex.end();
        group.add(&flex);
        
        (import_path_input, browse_import_btn, auto_extract_checkbox, auto_move_checkbox)
    }
    
    fn create_search_tab(group: &mut Group) -> (CheckButton, IntInput) {
        let mut flex = Flex::new(20, 50, 640, 400, None);
        flex.set_type(FlexType::Column);
        flex.set_spacing(20);
        
        // 搜索功能组
        let search_group = Group::new(0, 0, 640, 80, None);
        let mut search_label = Frame::new(0, 0, 200, 25, "搜索功能:");
        search_label.set_align(Align::Left | Align::Inside);
        
        let enable_fts_checkbox = CheckButton::new(0, 35, 300, 25, "启用全文搜索 (FTS)");
        enable_fts_checkbox.set_checked(true);
        
        search_group.end();
        flex.fixed(&search_group, 80);
        
        // 搜索限制组
        let limit_group = Group::new(0, 0, 640, 80, None);
        let mut limit_label = Frame::new(0, 0, 200, 25, "搜索结果限制:");
        limit_label.set_align(Align::Left | Align::Inside);
        
        let mut limit_flex = Flex::new(0, 30, 640, 40, None);
        limit_flex.set_type(FlexType::Row);
        limit_flex.set_spacing(10);
        
        let mut max_results_input = IntInput::new(0, 0, 100, 30, None);
        max_results_input.set_value("1000");
        max_results_input.set_color(Color::White);
        limit_flex.fixed(&max_results_input, 100);
        
        let mut results_label = Frame::new(0, 0, 150, 30, "最大结果数");
        results_label.set_align(Align::Left | Align::Inside);
        limit_flex.fixed(&results_label, 150);
        
        // 填充空间
        let spacer = Frame::new(0, 0, 0, 0, None);
        limit_flex.fixed(&spacer, 390);
        
        limit_flex.end();
        limit_group.end();
        flex.fixed(&limit_group, 80);
        
        // 搜索说明
        let mut info_frame = Frame::new(0, 0, 640, 120,
            "搜索设置:\n\n• 全文搜索可以搜索文档内容，而不仅是文件名\n• 搜索结果限制可以提高性能\n• 支持关键词、标签、作者等多种搜索方式\n\n注意: 禁用 FTS 将只能按文件名和元数据搜索");
        info_frame.set_align(Align::Left | Align::Top | Align::Inside);
        info_frame.set_label_color(Color::from_rgb(108, 117, 125));
        flex.fixed(&info_frame, 120);
        
        flex.end();
        group.add(&flex);
        
        (enable_fts_checkbox, max_results_input)
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
        self.modified = false;
        
        // 更新界面显示
        if let Some(path) = &config_path {
            self.config_path_output.set_value(&path.to_string_lossy());
        } else {
            self.config_path_output.set_value("No configuration file loaded");
        }
        
        // 设置数据库路径
        self.db_path_output.set_value(&config.database.path.to_string_lossy());
        
        // 设置导入路径
        self.import_path_input.set_value(&config.import.paths.storage_dir.to_string_lossy());
        
        // 设置导入选项
        self.auto_extract_checkbox.set_checked(config.import.metadata.prefer_json);
        self.auto_move_checkbox.set_checked(false); // 配置中暂时没有这个选项
        
        // 设置搜索选项
        self.enable_fts_checkbox.set_checked(config.search.enable_fts);
        self.max_results_input.set_value(&config.search.default_limit.to_string());
    }
    
    fn setup_callbacks(&mut self) {
        // 配置文件浏览按钮
        {
            let sender = self.event_sender.clone();
            let mut output = self.config_path_output.clone();
            let mut dialog = self.clone_for_callback();
            self.browse_config_btn.set_callback(move |_| {
                if let Some(path) = Self::browse_config_file() {
                    output.set_value(&path.to_string_lossy());
                    
                    // 立即加载选中的配置文件
                    if let Err(e) = dialog.load_config_file(&path) {
                        eprintln!("Failed to load config file: {}", e);
                        fltk::dialog::alert_default(&format!("Failed to load config file: {}", e));
                    } else {
                        println!("Loaded config file: {}", path.display());
                        
                        // 发送配置更新事件
                        let _ = sender.send(AppEvent::ConfigUpdated(path.clone()));
                    }
                }
            });
        }
        
        // 编辑配置文件按钮
        {
            let path_output = self.config_path_output.clone();
            self.edit_config_btn.set_callback(move |_| {
                let path_str = path_output.value();
                if !path_str.is_empty() && path_str != "No configuration file loaded" {
                    let path = PathBuf::from(path_str);
                    if let Err(e) = Self::edit_config_file(&path) {
                        eprintln!("Failed to open config file: {}", e);
                    }
                }
            });
        }
        
        // 重新加载配置按钮
        {
            let sender = self.event_sender.clone();
            self.reload_config_btn.set_callback(move |_| {
                let _ = sender.send(AppEvent::RefreshView);
            });
        }
        
        // 数据库浏览按钮
        {
            let mut output = self.db_path_output.clone();
            self.browse_db_btn.set_callback(move |_| {
                if let Some(path) = Self::browse_database_file() {
                    output.set_value(&path.to_string_lossy());
                }
            });
        }
        
        // 数据库备份按钮
        {
            let output = self.db_path_output.clone();
            self.backup_db_btn.set_callback(move |_| {
                let db_path = output.value();
                if !db_path.is_empty() {
                    Self::backup_database(&PathBuf::from(db_path));
                }
            });
        }
        
        // 重建索引按钮
        {
            let sender = self.event_sender.clone();
            self.rebuild_index_btn.set_callback(move |_| {
                // TODO: 实现重建索引功能
                println!("Rebuilding search index...");
            });
        }
        
        // 导入目录浏览按钮
        {
            let mut input = self.import_path_input.clone();
            self.browse_import_btn.set_callback(move |_| {
                if let Some(path) = Self::browse_directory() {
                    input.set_value(&path.to_string_lossy());
                }
            });
        }
        
        // 保存按钮
        {
            let mut dialog = self.clone_for_callback();
            self.save_btn.set_callback(move |_| {
                if let Err(e) = dialog.save_settings() {
                    eprintln!("Failed to save settings: {}", e);
                    fltk::dialog::alert_default(&format!("保存设置失败: {}", e));
                } else {
                    dialog.window.hide();
                }
            });
        }
        
        // 重置按钮
        {
            let mut dialog = self.clone_for_callback();
            self.reset_btn.set_callback(move |_| {
                if fltk::dialog::choice2_default("重置所有设置到默认值?", "取消", "重置", "") == Some(1) {
                    dialog.reset_to_defaults();
                }
            });
        }
        
        // 取消按钮
        {
            let mut window = self.window.clone();
            self.cancel_btn.set_callback(move |_| {
                window.hide();
            });
        }
    }
    
    fn clone_for_callback(&self) -> Self {
        // 创建一个用于回调的简化副本
        Self {
            window: self.window.clone(),
            config_path_output: self.config_path_output.clone(),
            browse_config_btn: self.browse_config_btn.clone(),
            edit_config_btn: self.edit_config_btn.clone(),
            reload_config_btn: self.reload_config_btn.clone(),
            db_path_output: self.db_path_output.clone(),
            browse_db_btn: self.browse_db_btn.clone(),
            backup_db_btn: self.backup_db_btn.clone(),
            rebuild_index_btn: self.rebuild_index_btn.clone(),
            import_path_input: self.import_path_input.clone(),
            browse_import_btn: self.browse_import_btn.clone(),
            auto_extract_checkbox: self.auto_extract_checkbox.clone(),
            auto_move_checkbox: self.auto_move_checkbox.clone(),
            enable_fts_checkbox: self.enable_fts_checkbox.clone(),
            max_results_input: self.max_results_input.clone(),
            save_btn: self.save_btn.clone(),
            cancel_btn: self.cancel_btn.clone(),
            reset_btn: self.reset_btn.clone(),
            current_config: self.current_config.clone(),
            config_path: self.config_path.clone(),
            modified: self.modified,
            event_sender: self.event_sender.clone(),
        }
    }
    
    fn save_settings(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut config) = self.current_config {
            // 更新数据库路径
            let db_path = self.db_path_output.value();
            if !db_path.is_empty() {
                config.database.path = PathBuf::from(db_path);
            }
            
            // 更新导入路径
            let import_path = self.import_path_input.value();
            if !import_path.is_empty() {
                config.import.paths.storage_dir = PathBuf::from(import_path);
            }
            
            // 更新导入选项
            config.import.metadata.prefer_json = self.auto_extract_checkbox.is_checked();
            
            // 保存到文件
            if let Some(config_path) = &self.config_path {
                let config_content = toml::to_string_pretty(config)?;
                fs::write(config_path, config_content)?;
                println!("Settings saved to: {}", config_path.display());
            }
        }
        
        Ok(())
    }
    
    fn reset_to_defaults(&mut self) {
        // 重置到默认值
        self.db_path_output.set_value("tagbox.db");
        self.import_path_input.set_value("./documents");
        self.auto_extract_checkbox.set_checked(true);
        self.auto_move_checkbox.set_checked(false);
        self.enable_fts_checkbox.set_checked(true);
        self.max_results_input.set_value("1000");
        self.modified = true;
    }
    
    fn load_config_file(&mut self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        // 使用 tagbox_core 加载配置
        let rt = tokio::runtime::Runtime::new()?;
        let config = rt.block_on(async {
            tagbox_core::load_config(path).await
        })?;
        
        // 更新当前配置
        self.current_config = Some(config.clone());
        self.config_path = Some(path.clone());
        
        // 更新界面显示
        self.load_config(config, Some(path.clone()));
        
        println!("Config loaded from: {}", path.display());
        Ok(())
    }
    
    fn browse_config_file() -> Option<PathBuf> {
        let mut dialog = NativeFileChooser::new(FileDialogType::BrowseFile);
        dialog.set_title("选择配置文件");
        dialog.set_filter("TOML Files\t*.toml");
        dialog.show();
        
        let filename = dialog.filename();
        if filename.to_string_lossy().is_empty() {
            None
        } else {
            Some(filename)
        }
    }
    
    fn browse_database_file() -> Option<PathBuf> {
        let mut dialog = NativeFileChooser::new(FileDialogType::BrowseSaveFile);
        dialog.set_title("选择数据库文件");
        dialog.set_filter("Database Files\t*.db\t*.sqlite");
        dialog.show();
        
        let filename = dialog.filename();
        if filename.to_string_lossy().is_empty() {
            None
        } else {
            Some(filename)
        }
    }
    
    fn browse_directory() -> Option<PathBuf> {
        let mut dialog = NativeFileChooser::new(FileDialogType::BrowseDir);
        dialog.set_title("选择目录");
        dialog.show();
        
        let filename = dialog.filename();
        if filename.to_string_lossy().is_empty() {
            None
        } else {
            Some(filename)
        }
    }
    
    fn edit_config_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // 尝试用系统默认编辑器打开配置文件
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(["/C", "start", "", path.to_str().unwrap_or("")])
                .spawn()?;
        }
        
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(path)
                .spawn()?;
        }
        
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(path)
                .spawn()?;
        }
        
        Ok(())
    }
    
    fn backup_database(db_path: &Path) {
        if !db_path.exists() {
            fltk::dialog::alert_default("数据库文件不存在");
            return;
        }
        
        let backup_name = format!("{}.backup.{}", 
            db_path.to_string_lossy(), 
            chrono::Local::now().format("%Y%m%d_%H%M%S"));
        let backup_path = db_path.parent().unwrap_or(Path::new(".")).join(backup_name);
        
        match fs::copy(db_path, &backup_path) {
            Ok(_) => {
                fltk::dialog::message_default(&format!("数据库备份成功:\n{}", backup_path.display()));
            },
            Err(e) => {
                fltk::dialog::alert_default(&format!("备份失败: {}", e));
            }
        }
    }
}