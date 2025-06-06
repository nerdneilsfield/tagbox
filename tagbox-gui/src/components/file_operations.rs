use fltk::{
    prelude::*,
    menu::{MenuButton, MenuFlag},
    enums::Color,
};
use std::sync::mpsc::Sender;
use std::path::Path;
use tagbox_core::types::FileEntry;
use crate::state::AppEvent;
use crate::utils::{copy_to_clipboard, open_folder, open_file};

pub struct FileOperationsMenu {
    menu: MenuButton,
    current_file: Option<FileEntry>,
    event_sender: Sender<AppEvent>,
}

impl FileOperationsMenu {
    pub fn new(x: i32, y: i32, w: i32, h: i32, event_sender: Sender<AppEvent>) -> Self {
        let mut menu = MenuButton::new(x, y, w, h, "⋮");
        menu.set_color(Color::from_rgb(248, 249, 250));
        menu.set_selection_color(Color::from_rgb(230, 240, 255));
        
        // 添加菜单项
        menu.add_choice("Open File");
        menu.add_choice("Show in Folder");
        menu.add_choice("Copy Path");
        menu.add_choice("Copy Name");
        menu.add_choice("");  // 分隔符
        menu.add_choice("Edit Metadata");
        menu.add_choice("Duplicate File");
        menu.add_choice("");  // 分隔符
        menu.add_choice("Move to Trash");
        menu.add_choice("Permanently Delete");
        
        // TODO: 设置分隔符和菜单项状态 - 需要更好的FLTK API支持
        
        Self {
            menu,
            current_file: None,
            event_sender,
        }
    }
    
    pub fn set_file(&mut self, file: FileEntry) {
        self.current_file = Some(file);
        
        // 根据文件状态启用/禁用菜单项
        self.update_menu_state();
        
        // 设置菜单回调
        self.set_menu_callbacks();
    }
    
    fn update_menu_state(&mut self) {
        if let Some(file) = &self.current_file {
            // 检查文件是否存在来决定启用哪些菜单项
            let file_exists = file.path.exists();
            
            // TODO: 启用/禁用菜单项 - 需要更好的FLTK API支持
            let _ = file_exists; // 避免未使用变量警告
        }
    }
    
    fn set_menu_callbacks(&mut self) {
        let file = self.current_file.clone();
        let sender = self.event_sender.clone();
        
        self.menu.set_callback(move |menu| {
            if let Some(ref file) = file {
                let choice = menu.value();
                
                match choice {
                    0 => { // Open File
                        if let Err(e) = Self::handle_open_file(&file.path) {
                            eprintln!("Failed to open file: {}", e);
                        }
                    },
                    1 => { // Show in Folder
                        if let Err(e) = Self::handle_show_in_folder(&file.path) {
                            eprintln!("Failed to show in folder: {}", e);
                        }
                    },
                    2 => { // Copy Path
                        if let Err(e) = Self::handle_copy_path(&file.path) {
                            eprintln!("Failed to copy path: {}", e);
                        }
                    },
                    3 => { // Copy Name
                        if let Err(e) = Self::handle_copy_name(file) {
                            eprintln!("Failed to copy name: {}", e);
                        }
                    },
                    5 => { // Edit Metadata
                        let _ = sender.send(AppEvent::FileEdit(file.id.clone()));
                    },
                    6 => { // Duplicate File
                        Self::handle_duplicate_file(file);
                    },
                    8 => { // Move to Trash
                        if Self::confirm_delete("move to trash") {
                            let _ = sender.send(AppEvent::DeleteFile);
                        }
                    },
                    9 => { // Permanently Delete
                        if Self::confirm_delete("permanently delete") {
                            // TODO: 实现永久删除
                        }
                    },
                    _ => {}
                }
            }
        });
    }
    
    fn handle_open_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        open_file(path)
    }
    
    fn handle_show_in_folder(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        open_folder(path)
    }
    
    fn handle_copy_path(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        copy_to_clipboard(&path.to_string_lossy())
    }
    
    fn handle_copy_name(file: &FileEntry) -> Result<(), Box<dyn std::error::Error>> {
        let name = if !file.title.is_empty() {
            &file.title
        } else {
            &file.original_filename
        };
        copy_to_clipboard(name)
    }
    
    fn handle_duplicate_file(file: &FileEntry) {
        // TODO: 实现文件复制功能
        println!("Duplicating file: {}", file.title);
    }
    
    fn confirm_delete(action: &str) -> bool {
        let choice = fltk::dialog::choice2_default(
            &format!("Are you sure you want to {} this file?", action),
            "Yes",
            "No",
            ""
        );
        choice == Some(0)
    }
    
    pub fn show_at_position(&mut self, x: i32, y: i32) {
        self.menu.resize(x, y, self.menu.w(), self.menu.h());
        self.menu.popup();
    }
    
    pub fn clear_file(&mut self) {
        self.current_file = None;
        // TODO: 禁用所有菜单项
    }
}

// 键盘快捷键处理
pub struct KeyboardShortcuts {
    event_sender: Sender<AppEvent>,
}

impl KeyboardShortcuts {
    pub fn new(event_sender: Sender<AppEvent>) -> Self {
        Self { event_sender }
    }
    
    pub fn handle_key_event(&self, key: fltk::enums::Key) -> bool {
        // 使用常量定义快捷键
        const CTRL_F: i32 = 0x40000000 | ('f' as i32);
        const CTRL_N: i32 = 0x40000000 | ('n' as i32);
        const CTRL_E: i32 = 0x40000000 | ('e' as i32);
        
        match key {
            k if k == fltk::enums::Key::from_i32(CTRL_F) => { // Ctrl+F
                // TODO: 聚焦搜索框
                true
            },
            k if k == fltk::enums::Key::from_i32(CTRL_N) => { // Ctrl+N
                // TODO: 新建文件/导入文件
                true
            },
            k if k == fltk::enums::Key::from_i32(CTRL_E) => { // Ctrl+E
                // TODO: 编辑当前选中文件
                true
            },
            fltk::enums::Key::Delete => { // Delete
                let _ = self.event_sender.send(AppEvent::DeleteFile);
                true
            },
            fltk::enums::Key::F5 => { // F5 - 刷新
                let _ = self.event_sender.send(AppEvent::RefreshView);
                true
            },
            _ => false
        }
    }
}