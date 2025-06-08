use fltk::{
    prelude::*,
    input::Input,
    button::Button,
    enums::{Color, FrameType},
    menu::MenuButton,
};
use std::sync::mpsc::Sender;
use crate::state::AppEvent;
use tagbox_core::config::AppConfig;

pub struct SearchBar {
    input: Input,
    advanced_btn: Button,
    suggestions_menu: MenuButton,
    event_sender: Sender<AppEvent>,
}

impl SearchBar {
    pub fn new(x: i32, y: i32, w: i32, h: i32, event_sender: Sender<AppEvent>) -> Self {
        let mut input = Input::new(x + 10, y + 10, w - 130, 30, None);
        input.set_frame(FrameType::FlatBox);
        input.set_color(Color::White);
        input.set_text_size(14);
        input.set_value("Search files, tags, authors...");
        input.set_text_color(Color::from_rgb(128, 128, 128));
        
        // 高级搜索按钮
        let mut advanced_btn = Button::new(x + w - 110, y + 10, 100, 30, "Advanced");
        advanced_btn.set_color(Color::from_rgb(108, 117, 125));
        advanced_btn.set_label_color(Color::White);
        
        // 搜索建议菜单（隐藏）
        let suggestions_menu = MenuButton::new(x + 10, y + 45, w - 130, 0, None);
        
        // 设置回调 - 只在回车时搜索
        let sender_clone = event_sender.clone();
        input.set_callback(move |input| {
            // 回车键搜索
            if fltk::app::event_key() == fltk::enums::Key::Enter {
                let query = input.value();
                if !query.is_empty() && !query.contains("Search files, tags, authors...") {
                    println!("Search query: {}", query);
                    let _ = sender_clone.send(AppEvent::SearchQuery(query));
                }
            }
        });
        
        // 处理焦点事件
        input.handle(move |input, event| {
            match event {
                fltk::enums::Event::Focus => {
                    // 获得焦点时清除占位符
                    let value = input.value();
                    if value.contains("Search files, tags, authors...") {
                        input.set_value("");
                        input.set_text_color(Color::Black);
                    }
                    false
                }
                fltk::enums::Event::Unfocus => {
                    // 失去焦点时恢复占位符
                    let value = input.value();
                    if value.trim().is_empty() {
                        input.set_value("Search files, tags, authors...");
                        input.set_text_color(Color::from_rgb(128, 128, 128));
                    }
                    false
                }
                _ => false,
            }
        });
        
        let sender_clone = event_sender.clone();
        advanced_btn.set_callback(move |_| {
            // 打开高级搜索对话框
            println!("Opening advanced search dialog...");
            let _ = sender_clone.send(AppEvent::OpenAdvancedSearch);
        });
        
        Self {
            input,
            advanced_btn,
            suggestions_menu,
            event_sender,
        }
    }
    
    pub fn get_query(&self) -> String {
        let value = self.input.value();
        if value.contains("Search files, tags, authors...") {
            String::new()
        } else {
            value
        }
    }
    
    pub fn set_placeholder(&mut self, placeholder: &str) {
        if self.get_query().is_empty() {
            self.input.set_value(placeholder);
            self.input.set_text_color(Color::from_rgb(128, 128, 128));
        }
    }
    
    pub fn clear(&mut self) {
        self.input.set_value("");
        self.set_placeholder("Search files, tags, authors...");
    }
    
    pub fn set_loading(&mut self, loading: bool) {
        if loading {
            self.input.deactivate();
        } else {
            self.input.activate();
        }
    }
    
    // 显示搜索建议
    pub fn show_suggestions(&mut self, suggestions: Vec<String>) {
        self.suggestions_menu.clear();
        
        if suggestions.is_empty() {
            self.suggestions_menu.hide();
            return;
        }
        
        // 添加建议项
        for suggestion in suggestions.iter().take(10) {
            self.suggestions_menu.add_choice(suggestion);
        }
        
        // 设置位置和大小
        let menu_height = std::cmp::min(suggestions.len() * 25, 250) as i32;
        self.suggestions_menu.resize(
            self.input.x(),
            self.input.y() + self.input.height(),
            self.input.width(),
            menu_height
        );
        
        self.suggestions_menu.show();
    }
    
    pub fn hide_suggestions(&mut self) {
        self.suggestions_menu.hide();
    }
    
    // 启用实时搜索建议 - 简化版本，不自动搜索
    pub fn enable_live_suggestions(&mut self, _config: AppConfig) {
        // 暂时禁用自动搜索，避免输入被打断
        // 用户需要按回车键来搜索
    }
    
    // 执行搜索
    pub fn execute_search(&self) {
        let query = self.get_query();
        if !query.is_empty() {
            println!("Executing manual search: {}", query);
            let _ = self.event_sender.send(AppEvent::SearchQuery(query));
        } else {
            // 如果查询为空，显示所有文件
            println!("Showing all files");
            let _ = self.event_sender.send(AppEvent::SearchQuery("*".to_string()));
        }
    }
    
    // 聚焦到搜索框
    pub fn focus(&mut self) {
        self.input.take_focus().unwrap_or(());
        // 选中所有文本
        let value = self.input.value();
        if !value.contains("Search files, tags, authors...") {
            self.input.set_position(0).unwrap_or(());
            self.input.set_mark(value.len() as i32).unwrap_or(());
        }
    }
    
    // 设置查询内容
    pub fn set_query(&mut self, query: &str) {
        self.input.set_value(query);
        self.input.set_text_color(Color::Black);
    }
    
    // 获取input组件引用
    pub fn widget(&mut self) -> &mut Input {
        &mut self.input
    }
}