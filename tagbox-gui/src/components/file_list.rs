use fltk::{
    prelude::*,
    enums::{Color, Event},
    group::Group,
    menu::MenuButton,
    app::MouseButton,
};
use fltk_table::{SmartTable, TableOpts};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use tagbox_core::types::{FileEntry, SearchResult};
use crate::state::AppEvent;

pub struct FileList {
    container: Group,
    table: SmartTable,
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
        
        // 创建 SmartTable
        let mut table = SmartTable::new(x, y, w, h, None);
        
        // 设置表格选项
        let opts = TableOpts {
            rows: 0,
            cols: 5, // Title, Authors, Year, Tags, Category
            editable: false,
            ..Default::default()
        };
        table.set_opts(opts);
        
        // 设置列标题
        table.set_col_header_value(0, "Title");
        table.set_col_header_value(1, "Authors");
        table.set_col_header_value(2, "Year");
        table.set_col_header_value(3, "Tags");
        table.set_col_header_value(4, "Category");
        
        // 设置列宽
        table.set_col_width(0, 300); // Title
        table.set_col_width(1, 200); // Authors
        table.set_col_width(2, 60);  // Year
        table.set_col_width(3, 150); // Tags
        table.set_col_width(4, 150); // Category
        
        // 设置表格样式
        table.set_col_header_height(25);
        table.set_row_height_all(25);
        table.set_selection_color(Color::from_rgb(230, 240, 255));
        
        container.end();
        
        let files = Arc::new(Mutex::new(Vec::new()));
        
        let mut file_list = Self {
            container,
            table,
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
        let files_ref = Arc::clone(&self.files);
        
        // 选择回调
        self.table.set_callback(move |table| {
            let (row, _, _, _) = table.get_selection();
            
            if row >= 0 {
                let _ = sender.send(AppEvent::FileSelected(format!("index:{}", row)));
            }
        });
        
        // 右键菜单处理
        let files_ref_menu = Arc::clone(&self.files);
        self.table.handle(move |table, event| {
            match event {
                Event::Push => {
                    if fltk::app::event_mouse_button() == MouseButton::Right {
                        // 获取鼠标位置对应的行
                        let mouse_y = fltk::app::event_y() - table.y();
                        let row = mouse_y / table.row_height(0);
                        
                        let files = files_ref_menu.lock().unwrap();
                        if row >= 0 && (row as usize) < files.len() {
                            // 选中该行
                            table.set_selection(row, 0, row, 4);
                            table.redraw();
                            
                            // 显示右键菜单
                            Self::show_context_menu(row as usize, &sender_menu);
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
        
        // 更新表格行数
        self.table.set_rows(files.len() as i32);
        
        if files.is_empty() {
            // 如果没有文件，显示提示信息
            self.table.set_rows(1);
            self.table.set_cell_value(0, 0, "No files found. Try a different search or import some files.");
            for col in 1..5 {
                self.table.set_cell_value(0, col, "");
            }
            self.table.deactivate();
            return Ok(());
        }
        
        // 激活表格
        self.table.activate();
        
        // 填充表格数据
        for (row, file) in files.iter().enumerate() {
            // Title
            let display_title = if file.title.is_empty() {
                &file.original_filename
            } else {
                &file.title
            };
            let title = Self::truncate_string(display_title, 40);
            self.table.set_cell_value(row as i32, 0, &title);
            
            // Authors
            let authors_str = if file.authors.is_empty() {
                "Unknown".to_string()
            } else {
                Self::truncate_string(&file.authors.join(", "), 25)
            };
            self.table.set_cell_value(row as i32, 1, &authors_str);
            
            // Year
            let year_str = file.year.map(|y| y.to_string()).unwrap_or_else(|| "----".to_string());
            self.table.set_cell_value(row as i32, 2, &year_str);
            
            // Tags
            let tags_str = match file.tags.len() {
                0 => "No tags".to_string(),
                1 => file.tags[0].clone(),
                n => format!("{} tags", n),
            };
            self.table.set_cell_value(row as i32, 3, &tags_str);
            
            // Category
            let category_str = if file.category1.is_empty() {
                "Uncategorized".to_string()
            } else {
                file.category1.clone()
            };
            self.table.set_cell_value(row as i32, 4, &category_str);
        }
        
        self.table.redraw();
        println!("Loaded {} files into table", files.len());
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
        self.table.set_rows(0);
        self.table.redraw();
    }
    
    pub fn get_selected_file(&self) -> Option<FileEntry> {
        let (row, _, _, _) = self.table.get_selection();
        if row >= 0 {
            let files = self.files.lock().unwrap();
            files.get(row as usize).cloned()
        } else {
            self.selected_index.and_then(|index| {
                let files = self.files.lock().unwrap();
                files.get(index).cloned()
            })
        }
    }
    
    pub fn select_file(&mut self, file_id: &str) {
        let files = self.files.lock().unwrap();
        for (index, file) in files.iter().enumerate() {
            if file.id == file_id {
                self.selected_index = Some(index);
                self.table.set_selection(index as i32, 0, index as i32, 4);
                self.table.redraw();
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
            self.table.set_selection(index as i32, 0, index as i32, 4);
            self.table.redraw();
            if let Some(file) = files.get(index) {
                println!("Selected file by index: {} (index: {})", file.title, index);
            }
        }
    }
    
    // 获取当前选中的文件
    pub fn get_current_selection(&self) -> Option<FileEntry> {
        let (row, _, _, _) = self.table.get_selection();
        if row >= 0 {
            let files = self.files.lock().unwrap();
            files.get(row as usize).cloned()
        } else {
            None
        }
    }
    
    // 获取当前文件列表
    pub fn get_current_files(&self) -> Vec<FileEntry> {
        let files = self.files.lock().unwrap();
        files.clone()
    }
    
    pub fn refresh(&mut self) {
        self.table.redraw();
    }
    
    pub fn set_loading(&mut self, loading: bool) {
        if loading {
            self.table.deactivate();
        } else {
            self.table.activate();
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
            self.table.set_rows(1);
            self.table.set_cell_value(0, 0, "Drag files here to import...");
            for col in 1..5 {
                self.table.set_cell_value(0, col, "");
            }
        } else {
            self.clear();
        }
        self.table.redraw();
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