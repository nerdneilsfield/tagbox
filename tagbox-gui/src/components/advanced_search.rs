use fltk::{
    prelude::*,
    window::Window,
    group::{Flex, FlexType, Group},
    input::{Input, IntInput},
    button::{Button, CheckButton},
    menu::Choice,
    enums::{Color, FrameType},
    frame::Frame,
    text::{TextEditor, TextBuffer},
};
use std::sync::mpsc::Sender;
use tagbox_core::{config::AppConfig, types::SearchOptions};
use crate::state::AppEvent;

pub struct AdvancedSearchDialog {
    window: Window,
    
    // 搜索条件
    title_input: Input,
    authors_input: Input,
    year_from_input: IntInput,
    year_to_input: IntInput,
    publisher_input: Input,
    source_input: Input,
    tags_input: Input,
    content_editor: TextEditor,
    content_buffer: TextBuffer,
    
    // 分类筛选
    category_l1: Choice,
    category_l2: Choice,
    category_l3: Choice,
    
    // 高级选项
    exact_match_checkbox: CheckButton,
    case_sensitive_checkbox: CheckButton,
    include_content_checkbox: CheckButton,
    
    // 操作按钮
    search_btn: Button,
    clear_btn: Button,
    cancel_btn: Button,
    
    event_sender: Sender<AppEvent>,
}

impl AdvancedSearchDialog {
    pub fn new(event_sender: Sender<AppEvent>) -> Self {
        let mut window = Window::new(200, 200, 750, 700, "Advanced Search");
        window.make_modal(true);
        window.set_color(Color::from_rgb(248, 249, 250));
        
        let padding = 20;
        let field_height = 30;
        let label_height = 20;
        let spacing = 8;
        let y = padding;
        
        // 主容器
        let mut main_flex = Flex::new(padding, y, 710, 640, None);
        main_flex.set_type(FlexType::Column);
        main_flex.set_spacing(spacing);
        
        // 基本搜索条件区域
        let mut basic_group = Group::new(0, 0, 710, 220, "Basic Search Criteria");
        basic_group.set_frame(FrameType::BorderBox);
        basic_group.set_color(Color::White);
        basic_group.set_label_color(Color::from_rgb(51, 51, 51));
        basic_group.set_label_size(14);
        
        let mut y_basic = 20;
        
        // 标题搜索
        let _title_label = Frame::new(padding, y_basic, 100, label_height, "Title:");
        y_basic += label_height + 5;
        let mut title_input = Input::new(padding, y_basic, 670, field_height, None);
        title_input.set_color(Color::White);
        title_input.set_tooltip("Search in file titles");
        y_basic += field_height + spacing;
        
        // 作者搜索
        let _authors_label = Frame::new(padding, y_basic, 100, label_height, "Authors:");
        y_basic += label_height + 5;
        let mut authors_input = Input::new(padding, y_basic, 670, field_height, None);
        authors_input.set_color(Color::White);
        authors_input.set_tooltip("Search for specific authors (comma separated)");
        y_basic += field_height + spacing;
        
        // 年份范围
        let _year_label = Frame::new(padding, y_basic, 100, label_height, "Year Range:");
        y_basic += label_height + 5;
        
        let mut year_from_input = IntInput::new(padding, y_basic, 100, field_height, None);
        year_from_input.set_color(Color::White);
        year_from_input.set_tooltip("From year (e.g., 2020)");
        
        let _year_to_label = Frame::new(padding + 110, y_basic + 5, 20, 20, "to");
        
        let mut year_to_input = IntInput::new(padding + 140, y_basic, 100, field_height, None);
        year_to_input.set_color(Color::White);
        year_to_input.set_tooltip("To year (e.g., 2023)");
        
        // 出版商和来源 - 同一行
        let mut publisher_input = Input::new(padding + 260, y_basic, 200, field_height, None);
        publisher_input.set_color(Color::White);
        publisher_input.set_tooltip("Publisher name");
        
        let mut source_input = Input::new(padding + 470, y_basic, 200, field_height, None);
        source_input.set_color(Color::White);
        source_input.set_tooltip("Source or origin");
        
        y_basic += field_height + spacing;
        
        // 标签搜索
        let _tags_label = Frame::new(padding, y_basic, 100, label_height, "Tags:");
        y_basic += label_height + 5;
        let mut tags_input = Input::new(padding, y_basic, 670, field_height, None);
        tags_input.set_color(Color::White);
        tags_input.set_tooltip("Search for specific tags (comma separated)");
        
        basic_group.end();
        main_flex.fixed(&basic_group, 220);
        
        // 分类筛选区域
        let mut category_group = Group::new(0, 0, 710, 100, "Category Filter");
        category_group.set_frame(FrameType::BorderBox);
        category_group.set_color(Color::White);
        category_group.set_label_color(Color::from_rgb(51, 51, 51));
        category_group.set_label_size(14);
        
        let y_cat = 30;
        let _category_label = Frame::new(padding, y_cat, 100, label_height, "Categories:");
        
        let mut category_l1 = Choice::new(padding, y_cat + 25, 200, field_height, None);
        category_l1.add_choice("Any Category 1");
        category_l1.set_value(0);
        
        let mut category_l2 = Choice::new(padding + 220, y_cat + 25, 200, field_height, None);
        category_l2.add_choice("Any Category 2");
        category_l2.set_value(0);
        category_l2.deactivate();
        
        let mut category_l3 = Choice::new(padding + 440, y_cat + 25, 200, field_height, None);
        category_l3.add_choice("Any Category 3");
        category_l3.set_value(0);
        category_l3.deactivate();
        
        category_group.end();
        main_flex.fixed(&category_group, 100);
        
        // 内容搜索区域
        let mut content_group = Group::new(0, 0, 710, 120, "Content Search");
        content_group.set_frame(FrameType::BorderBox);
        content_group.set_color(Color::White);
        content_group.set_label_color(Color::from_rgb(51, 51, 51));
        content_group.set_label_size(14);
        
        let _content_label = Frame::new(padding, 25, 100, label_height, "Full Text:");
        let mut content_editor = TextEditor::new(padding, 50, 670, 60, None);
        content_editor.set_color(Color::White);
        content_editor.set_tooltip("Search within file content");
        let content_buffer = TextBuffer::default();
        content_editor.set_buffer(Some(content_buffer.clone()));
        
        content_group.end();
        main_flex.fixed(&content_group, 120);
        
        // 高级选项区域
        let mut options_group = Group::new(0, 0, 710, 80, "Search Options");
        options_group.set_frame(FrameType::BorderBox);
        options_group.set_color(Color::White);
        options_group.set_label_color(Color::from_rgb(51, 51, 51));
        options_group.set_label_size(14);
        
        let mut exact_match_checkbox = CheckButton::new(padding, 30, 200, 25, "Exact phrase match");
        exact_match_checkbox.set_tooltip("Match exact phrases instead of individual words");
        
        let mut case_sensitive_checkbox = CheckButton::new(padding + 220, 30, 200, 25, "Case sensitive");
        case_sensitive_checkbox.set_tooltip("Match case when searching");
        
        let mut include_content_checkbox = CheckButton::new(padding, 55, 300, 25, "Include full-text content search");
        include_content_checkbox.set_tooltip("Search within file content (slower but more comprehensive)");
        include_content_checkbox.set_value(true); // 默认启用
        
        options_group.end();
        main_flex.fixed(&options_group, 80);
        
        // 按钮区域
        let mut button_flex = Flex::new(0, 0, 710, 40, None);
        button_flex.set_type(FlexType::Row);
        button_flex.set_spacing(10);
        
        let mut search_btn = Button::new(0, 0, 0, 0, "Search");
        search_btn.set_color(Color::from_rgb(0, 123, 255));
        search_btn.set_label_color(Color::White);
        search_btn.set_label_size(14);
        
        let mut clear_btn = Button::new(0, 0, 0, 0, "Clear All");
        clear_btn.set_color(Color::from_rgb(108, 117, 125));
        clear_btn.set_label_color(Color::White);
        clear_btn.set_label_size(14);
        
        let mut cancel_btn = Button::new(0, 0, 0, 0, "Cancel");
        cancel_btn.set_color(Color::from_rgb(220, 53, 69));
        cancel_btn.set_label_color(Color::White);
        cancel_btn.set_label_size(14);
        
        button_flex.end();
        main_flex.fixed(&button_flex, 40);
        
        main_flex.end();
        window.end();
        
        Self {
            window,
            title_input,
            authors_input,
            year_from_input,
            year_to_input,
            publisher_input,
            source_input,
            tags_input,
            content_editor,
            content_buffer,
            category_l1,
            category_l2,
            category_l3,
            exact_match_checkbox,
            case_sensitive_checkbox,
            include_content_checkbox,
            search_btn,
            clear_btn,
            cancel_btn,
            event_sender,
        }
    }
    
    pub fn show(&mut self) {
        self.window.show();
        self.setup_callbacks();
    }
    
    pub fn hide(&mut self) {
        self.window.hide();
    }
    
    pub fn shown(&self) -> bool {
        self.window.shown()
    }
    
    fn setup_callbacks(&mut self) {
        // 搜索按钮回调
        let sender_clone = self.event_sender.clone();
        self.search_btn.set_callback(move |btn| {
            // 发送高级搜索事件，使用默认的SearchOptions
            let options = SearchOptions {
                offset: 0,
                limit: 100,
                sort_by: Some("updated_at".to_string()),
                sort_direction: Some("DESC".to_string()),
                include_deleted: false,
            };
            let _ = sender_clone.send(AppEvent::AdvancedSearch(options));
            if let Some(mut window) = btn.window() {
                window.hide();
            }
        });
        
        // 清空按钮回调
        self.clear_btn.set_callback(move |_| {
            // 发送清空事件
            // 由于闭包限制，实际清空操作需要在父组件中处理
        });
        
        // 取消按钮回调
        let sender_clone = self.event_sender.clone();
        self.cancel_btn.set_callback(move |btn| {
            if let Some(mut window) = btn.window() {
                window.hide();
            }
        });
        
        // 分类级联选择
        self.category_l1.set_callback(move |choice| {
            let selected = choice.value();
            if selected > 0 {
                // TODO: 加载二级分类
            }
        });
        
        // Escape 键处理
        let sender_escape = self.event_sender.clone();
        self.window.set_callback(move |win| {
            if fltk::app::event() == fltk::enums::Event::KeyDown 
                && fltk::app::event_key() == fltk::enums::Key::Escape {
                win.hide();
            }
        });
    }
    
    pub async fn populate_categories(&mut self, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        // 获取所有分类数据并填充下拉框
        let search_result = tagbox_core::search_files_advanced("", None, config).await?;
        
        let mut level1_categories = std::collections::HashSet::new();
        let mut level2_categories = std::collections::HashMap::new();
        let mut level3_categories = std::collections::HashMap::new();
        
        for file in &search_result.entries {
            level1_categories.insert(file.category1.clone());
            
            if let Some(cat2) = &file.category2 {
                level2_categories.entry(file.category1.clone())
                    .or_insert_with(|| std::collections::HashSet::new())
                    .insert(cat2.clone());
            }
            
            if let Some(cat3) = &file.category3 {
                let key = format!("{}:{}", file.category1, file.category2.as_ref().unwrap_or(&"".to_string()));
                level3_categories.entry(key)
                    .or_insert_with(|| std::collections::HashSet::new())
                    .insert(cat3.clone());
            }
        }
        
        // 填充一级分类下拉框
        self.category_l1.clear();
        self.category_l1.add_choice("Any Category 1");
        
        let mut sorted_categories: Vec<_> = level1_categories.into_iter().collect();
        sorted_categories.sort();
        
        for category in sorted_categories {
            self.category_l1.add_choice(&category);
        }
        
        Ok(())
    }
    
    pub fn collect_search_options(&self) -> SearchOptions {
        // 使用实际的SearchOptions结构体
        SearchOptions {
            offset: 0,
            limit: 100,
            sort_by: Some("updated_at".to_string()),
            sort_direction: Some("DESC".to_string()),
            include_deleted: false,
        }
    }
    
    pub fn build_advanced_query(&self) -> String {
        let mut query_parts = Vec::new();
        
        // 标题搜索
        let title = self.title_input.value();
        if !title.trim().is_empty() {
            if self.exact_match_checkbox.is_checked() {
                query_parts.push(format!("\"{}\" in title", title));
            } else {
                query_parts.push(format!("{} in title", title));
            }
        }
        
        // 作者搜索
        let authors = self.authors_input.value();
        if !authors.trim().is_empty() {
            for author in authors.split(',') {
                let author = author.trim();
                if !author.is_empty() {
                    if self.exact_match_checkbox.is_checked() {
                        query_parts.push(format!("\"{}\" in authors", author));
                    } else {
                        query_parts.push(format!("{} in authors", author));
                    }
                }
            }
        }
        
        // 年份范围
        let year_from = self.year_from_input.value();
        let year_to = self.year_to_input.value();
        
        if !year_from.is_empty() {
            if let Ok(year) = year_from.parse::<i32>() {
                query_parts.push(format!("year >= {}", year));
            }
        }
        
        if !year_to.is_empty() {
            if let Ok(year) = year_to.parse::<i32>() {
                query_parts.push(format!("year <= {}", year));
            }
        }
        
        // 出版商搜索
        let publisher = self.publisher_input.value();
        if !publisher.trim().is_empty() {
            if self.exact_match_checkbox.is_checked() {
                query_parts.push(format!("\"{}\" in publisher", publisher));
            } else {
                query_parts.push(format!("{} in publisher", publisher));
            }
        }
        
        // 来源搜索
        let source = self.source_input.value();
        if !source.trim().is_empty() {
            if self.exact_match_checkbox.is_checked() {
                query_parts.push(format!("\"{}\" in source", source));
            } else {
                query_parts.push(format!("{} in source", source));
            }
        }
        
        // 标签搜索
        let tags = self.tags_input.value();
        if !tags.trim().is_empty() {
            for tag in tags.split(',') {
                let tag = tag.trim();
                if !tag.is_empty() {
                    if self.exact_match_checkbox.is_checked() {
                        query_parts.push(format!("\"{}\" in tags", tag));
                    } else {
                        query_parts.push(format!("{} in tags", tag));
                    }
                }
            }
        }
        
        // 分类搜索
        if self.category_l1.value() > 0 {
            if let Some(category) = self.category_l1.choice() {
                query_parts.push(format!("category1:{}", category));
            }
        }
        
        // 全文搜索
        if self.include_content_checkbox.is_checked() {
            let content = self.content_buffer.text();
            if !content.trim().is_empty() {
                if self.exact_match_checkbox.is_checked() {
                    query_parts.push(format!("\"{}\" in content", content));
                } else {
                    query_parts.push(format!("{} in content", content));
                }
            }
        }
        
        if query_parts.is_empty() {
            "*".to_string()
        } else {
            query_parts.join(" AND ")
        }
    }
    
    pub fn set_search_query(&mut self, query: &str) {
        // 解析简单查询并填充到对应字段
        if !query.is_empty() && query != "*" {
            self.title_input.set_value(query);
        }
    }
    
    pub fn clear_all_fields(&mut self) {
        self.title_input.set_value("");
        self.authors_input.set_value("");
        self.year_from_input.set_value("");
        self.year_to_input.set_value("");
        self.publisher_input.set_value("");
        self.source_input.set_value("");
        self.tags_input.set_value("");
        self.content_buffer.set_text("");
        self.category_l1.set_value(0);
        self.category_l2.set_value(0);
        self.category_l3.set_value(0);
        self.exact_match_checkbox.set_checked(false);
        self.case_sensitive_checkbox.set_checked(false);
        self.include_content_checkbox.set_checked(true);
    }
    
    pub fn validate_search_options(&self) -> Result<(), String> {
        // 检查年份范围是否有效
        let year_from = self.year_from_input.value().parse::<i32>().ok();
        let year_to = self.year_to_input.value().parse::<i32>().ok();
        
        if let (Some(from), Some(to)) = (year_from, year_to) {
            if from > to {
                return Err("Year range is invalid: 'from' year cannot be greater than 'to' year".to_string());
            }
        }
        
        // 检查是否至少有一个搜索条件
        if self.title_input.value().trim().is_empty() &&
           self.authors_input.value().trim().is_empty() &&
           self.publisher_input.value().trim().is_empty() &&
           self.source_input.value().trim().is_empty() &&
           self.tags_input.value().trim().is_empty() &&
           self.content_buffer.text().trim().is_empty() &&
           year_from.is_none() &&
           year_to.is_none() &&
           self.category_l1.value() <= 0 {
            return Err("Please provide at least one search criterion".to_string());
        }
        
        Ok(())
    }
}