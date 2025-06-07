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
        
        let mut table = Table::new(x, y, w, h, None);
        table.set_rows(0);
        table.set_cols(5); // Title, Authors, Year, Tags, Size
        table.set_col_header(true);
        table.set_col_resize(true);
        table.set_color(Color::White);
        table.set_selection_color(Color::from_rgb(230, 240, 255));
        
        // 设置列标题
        table.set_col_header_value(0, "Title");
        table.set_col_header_value(1, "Authors");
        table.set_col_header_value(2, "Year");
        table.set_col_header_value(3, "Tags");
        table.set_col_header_value(4, "Size");
        
        // 设置列宽
        table.set_col_width(0, w / 3);     // Title - 1/3
        table.set_col_width(1, w / 4);     // Authors - 1/4
        table.set_col_width(2, 60);       // Year - 固定宽度
        table.set_col_width(3, w / 4);     // Tags - 1/4
        table.set_col_width(4, 80);       // Size - 固定宽度
        
        // 创建操作菜单
        let operations_menu = FileOperationsMenu::new(0, 0, 100, 30, event_sender.clone());
        
        container.end();
        
        let mut file_list = Self {
            container,
            table,
            files: Vec::new(),
            selected_index: None,
            operations_menu,
            event_sender,
        };
        
        file_list.setup_callbacks();
        file_list
    }
    
    fn setup_callbacks(&mut self) {
        let sender = self.event_sender.clone();
        let files_ptr = &self.files as *const Vec<FileEntry>;
        
        self.table.draw_cell(move |table, ctx, row, col, x, y, w, h| {
            let files = unsafe { &*files_ptr };
            
            match ctx {
                fltk::table::TableContext::StartPage => fltk::draw::set_font(fltk::enums::Font::Helvetica, 12),
                fltk::table::TableContext::ColHeader => {
                    fltk::draw::push_clip(x, y, w, h);
                    fltk::draw::draw_box(fltk::enums::FrameType::ThinUpBox, x, y, w, h, Color::from_rgb(240, 240, 240));
                    fltk::draw::set_draw_color(Color::Black);
                    fltk::draw::draw_text2(&table.col_header_value(col), x, y, w, h, Align::Center);
                    fltk::draw::pop_clip();
                },
                fltk::table::TableContext::RowHeader => {
                    // 行标题（行号）
                    fltk::draw::push_clip(x, y, w, h);
                    fltk::draw::draw_box(fltk::enums::FrameType::ThinUpBox, x, y, w, h, Color::from_rgb(240, 240, 240));
                    fltk::draw::set_draw_color(Color::Black);
                    fltk::draw::draw_text2(&(row + 1).to_string(), x, y, w, h, Align::Center);
                    fltk::draw::pop_clip();
                },
                fltk::table::TableContext::Cell => {
                    if let Some(file) = files.get(row as usize) {
                        let selected = table.is_selected(row, col);
                        let bg_color = if selected {
                            Color::from_rgb(230, 240, 255)
                        } else {
                            Color::White
                        };
                        
                        fltk::draw::push_clip(x, y, w, h);
                        fltk::draw::draw_box(fltk::enums::FrameType::FlatBox, x, y, w, h, bg_color);
                        
                        let text = match col {
                            0 => { // Title
                                if file.title.is_empty() {
                                    &file.original_filename
                                } else {
                                    &file.title
                                }
                            },
                            1 => { // Authors
                                &file.authors.join(", ")
                            },
                            2 => { // Year
                                &file.year.map(|y| y.to_string()).unwrap_or_default()
                            },
                            3 => { // Tags
                                &file.tags.join(", ")
                            },
                            4 => { // Size
                                &Self::format_file_size(file.file_size_bytes)
                            },
                            _ => ""
                        };
                        
                        fltk::draw::set_draw_color(Color::Black);
                        fltk::draw::draw_text2(text, x + 5, y, w - 10, h, Align::Left | Align::Inside);
                        fltk::draw::pop_clip();
                    }
                },
                _ => {}
            }
        });
        
        // 表格事件处理
        self.table.handle(move |table, event| {
            match event {
                Event::Push => {
                    let row = table.callback_row();
                    let col = table.callback_col();
                    
                    if row >= 0 && col >= 0 {
                        table.set_selection(row, col, row, col);
                        let files = unsafe { &*files_ptr };
                        
                        if let Some(file) = files.get(row as usize) {
                            let _ = sender.send(AppEvent::FileSelected(file.id.clone()));
                        }
                    }
                    true
                },
                Event::Push if fltk::app::event_button() == fltk::app::MouseButton::Right => {
                    // 右键菜单
                    let row = table.callback_row();
                    if row >= 0 {
                        let files = unsafe { &*files_ptr };
                        if let Some(_file) = files.get(row as usize) {
                            // TODO: 显示右键菜单
                        }
                    }
                    true
                },
                Event::KeyDown => {
                    let key = fltk::app::event_key();
                    match key {
                        fltk::enums::Key::Delete => {
                            let _ = sender.send(AppEvent::DeleteFile("current".to_string()));
                            true
                        },
                        fltk::enums::Key::Enter => {
                            let row = table.callback_row();
                            if row >= 0 {
                                let files = unsafe { &*files_ptr };
                                if let Some(file) = files.get(row as usize) {
                                    let _ = sender.send(AppEvent::FileOpen(file.id.clone()));
                                }
                            }
                            true
                        },
                        _ => false
                    }
                },
                _ => false
            }
        });
    }
    
    pub async fn load_files(&mut self, search_result: SearchResult) -> Result<(), Box<dyn std::error::Error>> {
        self.files = search_result.entries;
        
        // 更新表格行数
        self.table.set_rows(self.files.len() as i32);
        self.table.redraw();
        
        Ok(())
    }
    
    pub fn clear(&mut self) {
        self.files.clear();
        self.selected_index = None;
        self.table.set_rows(0);
        self.table.redraw();
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
                self.table.set_selection(index as i32, 0, index as i32, 4);
                
                // 更新操作菜单
                self.operations_menu.set_file(file.clone());
                
                // 滚动到选中行
                self.table.set_top_row(index as i32);
                self.table.redraw();
                break;
            }
        }
    }
    
    pub fn refresh(&mut self) {
        self.table.redraw();
    }
    
    pub fn show_context_menu(&mut self, x: i32, y: i32) {
        if let Some(file) = self.get_selected_file() {
            self.operations_menu.set_file(file.clone());
            self.operations_menu.show_at_position(x, y);
        }
    }
    
    pub fn set_loading(&mut self, loading: bool) {
        if loading {
            self.table.deactivate();
        } else {
            self.table.activate();
        }
    }
    
    fn format_file_size(bytes: i64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
    
    // 排序功能
    pub fn sort_by_column(&mut self, column: i32, ascending: bool) {
        match column {
            0 => { // Title
                self.files.sort_by(|a, b| {
                    let a_title = if a.title.is_empty() { &a.original_filename } else { &a.title };
                    let b_title = if b.title.is_empty() { &b.original_filename } else { &b.title };
                    if ascending {
                        a_title.cmp(b_title)
                    } else {
                        b_title.cmp(a_title)
                    }
                });
            },
            1 => { // Authors
                self.files.sort_by(|a, b| {
                    let a_authors = a.authors.join(", ");
                    let b_authors = b.authors.join(", ");
                    if ascending {
                        a_authors.cmp(&b_authors)
                    } else {
                        b_authors.cmp(&a_authors)
                    }
                });
            },
            2 => { // Year
                self.files.sort_by(|a, b| {
                    if ascending {
                        a.year.cmp(&b.year)
                    } else {
                        b.year.cmp(&a.year)
                    }
                });
            },
            4 => { // Size
                self.files.sort_by(|a, b| {
                    if ascending {
                        a.file_size_bytes.cmp(&b.file_size_bytes)
                    } else {
                        b.file_size_bytes.cmp(&a.file_size_bytes)
                    }
                });
            },
            _ => {}
        }
        
        self.table.redraw();
    }
    
    // 过滤功能
    pub fn filter_files(&mut self, predicate: impl Fn(&FileEntry) -> bool) {
        // 这里可以实现客户端过滤功能
        // 目前保持简单，让服务端处理过滤
    }
    
    // 获取统计信息
    pub fn get_file_stats(&self) -> (usize, i64) {
        let count = self.files.len();
        let total_size = self.files.iter().map(|f| f.file_size_bytes).sum();
        (count, total_size)
    }
    
    // 获取容器引用（用于主窗口布局）
    pub fn widget(&mut self) -> &mut Group {
        &mut self.container
    }
}

// 表格列类型枚举
#[derive(Debug, Clone, Copy)]
pub enum FileListColumn {
    Title = 0,
    Authors = 1,
    Year = 2,
    Tags = 3,
    Size = 4,
}

impl FileListColumn {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Title => "Title",
            Self::Authors => "Authors", 
            Self::Year => "Year",
            Self::Tags => "Tags",
            Self::Size => "Size",
        }
    }
}