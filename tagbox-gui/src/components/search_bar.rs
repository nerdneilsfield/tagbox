use fltk::{
    prelude::*,
    input::Input,
    button::Button,
    menu::MenuButton,
    enums::{Color, Key},
};
use std::sync::mpsc::Sender;
use tagbox_core::config::AppConfig;
use crate::state::AppEvent;

pub struct SearchBar {
    input: Input,
    advanced_btn: Button,
    suggestions_menu: MenuButton,
    event_sender: Sender<AppEvent>,
}

impl SearchBar {
    pub fn new(
        x: i32, 
        y: i32, 
        w: i32, 
        h: i32,
        event_sender: Sender<AppEvent>
    ) -> Self {
        // 搜索输入框 (70% 宽度)
        let mut input = Input::new(x, y, (w as f32 * 0.7) as i32, h, None);
        input.set_value("Search (e.g. tag:Rust -tag:旧版)");
        input.set_text_color(Color::from_rgb(128, 128, 128));
        
        // 高级搜索按钮 (15% 宽度)
        let btn_x = x + (w as f32 * 0.7) as i32 + 10;
        let btn_w = (w as f32 * 0.15) as i32;
        let mut advanced_btn = Button::new(btn_x, y, btn_w, h, "Advanced");
        advanced_btn.set_color(Color::from_rgb(0, 123, 255));
        advanced_btn.set_label_color(Color::White);
        
        // 建议菜单 (隐藏状态)
        let mut suggestions_menu = MenuButton::new(x, y + h, (w as f32 * 0.7) as i32, 0, None);
        suggestions_menu.hide();
        
        // 设置回调
        let sender_clone = event_sender.clone();
        input.set_callback(move |input| {
            let key = fltk::app::event_key();
            if key == Key::Enter {
                let query = input.value();
                if !query.trim().is_empty() && !query.contains("Search (e.g.") {
                    let _ = sender_clone.send(AppEvent::SearchQuery(query));
                }
            }
        });
        
        let sender_clone = event_sender.clone();
        advanced_btn.set_callback(move |_| {
            // TODO: 打开高级搜索对话框
            // TODO: 实现高级搜索对话框
            // let _ = sender_clone.send(AppEvent::AdvancedSearch(search_options));
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
        if value.contains("Search (e.g.") {
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
        self.set_placeholder("Search (e.g. tag:Rust -tag:旧版)");
    }
    
    pub fn set_loading(&mut self, loading: bool) {
        if loading {
            self.input.set_value("Searching...");
            self.input.deactivate();
        } else {
            self.input.activate();
            if self.input.value() == "Searching..." {
                self.clear();
            }
        }
    }
    
    // 显示搜索建议
    pub fn show_suggestions(&mut self, suggestions: Vec<String>) {
        self.suggestions_menu.clear();
        
        if !suggestions.is_empty() {
            for suggestion in &suggestions {
                self.suggestions_menu.add_choice(&suggestion);
            }
            
            // 重新定位和显示建议菜单
            let (x, y, w, h) = (self.input.x(), self.input.y(), self.input.w(), self.input.h());
            self.suggestions_menu.resize(x, y + h, w, suggestions.len() as i32 * 25);
            self.suggestions_menu.show();
        } else {
            self.suggestions_menu.hide();
        }
    }
    
    pub fn hide_suggestions(&mut self) {
        self.suggestions_menu.hide();
    }
    
    // 启用实时搜索建议
    pub fn enable_live_suggestions(&mut self, _config: AppConfig) {
        let _sender = self.event_sender.clone();
        self.input.handle(move |input, event| {
            match event {
                fltk::enums::Event::KeyUp => {
                    let query = input.value();
                    if !query.is_empty() && !query.contains("Search (e.g.") && query.len() > 2 {
                        // 发送模糊搜索请求获取建议
                        // TODO: 实现建议功能
                        // let _ = sender.send(AppEvent::RequestSuggestions(query));
                    }
                    false
                }
                fltk::enums::Event::Focus => {
                    // 获得焦点时清除占位符
                    let value = input.value();
                    if value.contains("Search (e.g.") {
                        input.set_value("");
                        input.set_text_color(Color::Black);
                    }
                    false
                }
                fltk::enums::Event::Unfocus => {
                    // 失去焦点时恢复占位符
                    let value = input.value();
                    if value.trim().is_empty() {
                        input.set_value("Search (e.g. tag:Rust -tag:旧版)");
                        input.set_text_color(Color::from_rgb(128, 128, 128));
                    }
                    false
                }
                _ => false,
            }
        });
    }
}