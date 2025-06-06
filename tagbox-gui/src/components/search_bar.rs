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
        // 使用 Flex 布局管理器
        let mut container = fltk::group::Group::new(x, y, w, h, None);
        container.set_color(Color::from_rgb(248, 249, 250));
        
        let mut flex = fltk::group::Flex::new(x, y, w, h, None);
        flex.set_type(fltk::group::FlexType::Row);
        flex.set_spacing(10);
        flex.set_margin(5);
        
        // 搜索输入框 (占用大部分空间)
        let mut input = Input::new(0, 0, 0, h - 10, None);
        input.set_value("Search files, tags, authors...");
        input.set_text_color(Color::from_rgb(128, 128, 128));
        input.set_color(Color::White);
        input.set_text_size(13);
        // 让输入框占用大部分空间
        
        // 高级搜索按钮 (固定宽度)
        let mut advanced_btn = Button::new(0, 0, 0, h - 10, "Advanced");
        advanced_btn.set_color(Color::from_rgb(0, 123, 255));
        advanced_btn.set_label_color(Color::White);
        advanced_btn.set_label_size(12);
        flex.fixed(&advanced_btn, 80);
        
        flex.end();
        container.end();
        
        // 建议菜单 (隐藏状态)
        let mut suggestions_menu = MenuButton::new(x, y + h, w - 90, 0, None);
        suggestions_menu.hide();
        
        // 设置回调 - 支持回车键即时搜索
        let sender_clone = event_sender.clone();
        input.set_callback(move |input| {
            let key = fltk::app::event_key();
            if key == Key::Enter {
                let query = input.value();
                if !query.trim().is_empty() && !query.contains("Search files, tags, authors...") {
                    println!("Executing search: {}", query);
                    let _ = sender_clone.send(AppEvent::SearchQuery(query));
                }
            }
        });
        
        let sender_clone = event_sender.clone();
        advanced_btn.set_callback(move |_| {
            // 打开高级搜索对话框
            println!("Opening advanced search dialog...");
            // TODO: 创建并显示高级搜索对话框
            // let mut dialog = AdvancedSearchDialog::new(sender_clone.clone());
            // dialog.show();
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
    
    // 启用实时搜索建议和改进的用户体验
    pub fn enable_live_suggestions(&mut self, _config: AppConfig) {
        let sender = self.event_sender.clone();
        self.input.handle(move |input, event| {
            match event {
                fltk::enums::Event::KeyUp => {
                    let query = input.value();
                    if !query.is_empty() && !query.contains("Search files, tags, authors...") && query.len() > 2 {
                        // 实时搜索 - 当用户输入时立即触发搜索
                        println!("Live search triggered: {}", query);
                        let _ = sender.send(AppEvent::SearchQuery(query.clone()));
                    } else if query.trim().is_empty() || query.len() <= 2 {
                        // 清空搜索结果
                        println!("Clearing search results");
                        let _ = sender.send(AppEvent::SearchQuery("*".to_string()));
                    }
                    false
                }
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
    }
    
    // 执行搜索
    pub fn execute_search(&self) {
        let query = self.get_query();
        if !query.is_empty() {
            println!("Executing manual search: {}", query);
            let _ = self.event_sender.send(AppEvent::SearchQuery(query));
        } else {
            // 如果查询为空，显示所有文件
            let _ = self.event_sender.send(AppEvent::SearchQuery("*".to_string()));
        }
    }
    
    // 设置搜索查询
    pub fn set_query(&mut self, query: &str) {
        self.input.set_value(query);
        self.input.set_text_color(Color::Black);
    }
    
    // 获取搜索组件的引用（用于主窗口布局）
    pub fn widget(&mut self) -> &mut Input {
        &mut self.input
    }
}