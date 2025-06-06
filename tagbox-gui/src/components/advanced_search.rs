use fltk::{
    prelude::*,
    window::Window,
    group::{Flex, FlexType},
    input::{Input, IntInput},
    button::Button,
    menu::Choice,
    enums::Color,
    frame::Frame,
    text::{TextEditor, TextBuffer},
};
use std::sync::mpsc::Sender;
use tagbox_core::{config::AppConfig, types::SearchOptions};
use crate::state::AppEvent;

pub struct AdvancedSearchDialog {
    window: Window,
    
    // 搜索字段
    title_input: Input,
    author_input: Input,
    tag_input: Input,
    category_l1: Choice,
    category_l2: Choice,
    category_l3: Choice,
    year_from: IntInput,
    year_to: IntInput,
    publisher_input: Input,
    summary_input: TextEditor,
    summary_buffer: TextBuffer,
    
    // 按钮
    search_btn: Button,
    clear_btn: Button,
    cancel_btn: Button,
    
    event_sender: Sender<AppEvent>,
}

impl AdvancedSearchDialog {
    pub fn new(event_sender: Sender<AppEvent>) -> Self {
        let mut window = Window::new(200, 200, 500, 600, "Advanced Search");
        window.make_modal(true);
        window.set_color(Color::from_rgb(248, 249, 250));
        
        let padding = 15;
        let field_height = 30;
        let label_height = 20;
        let spacing = 10;
        let mut y = padding;
        
        // 标题搜索
        let _title_label = Frame::new(padding, y, 100, label_height, "Title contains:");
        y += label_height + 5;
        let mut title_input = Input::new(padding, y, 470, field_height, None);
        title_input.set_color(Color::White);
        y += field_height + spacing;
        
        // 作者搜索
        let _author_label = Frame::new(padding, y, 100, label_height, "Author:");
        y += label_height + 5;
        let mut author_input = Input::new(padding, y, 470, field_height, None);
        author_input.set_color(Color::White);
        y += field_height + spacing;
        
        // 标签搜索
        let _tag_label = Frame::new(padding, y, 100, label_height, "Tags (comma separated):");
        y += label_height + 5;
        let mut tag_input = Input::new(padding, y, 470, field_height, None);
        tag_input.set_color(Color::White);
        y += field_height + spacing;
        
        // 分类选择
        let _category_label = Frame::new(padding, y, 100, label_height, "Category:");
        y += label_height + 5;
        
        // 三个分类下拉框
        let mut category_l1 = Choice::new(padding, y, 150, field_height, None);
        category_l1.add_choice("All Categories");
        category_l1.set_value(0);
        
        let mut category_l2 = Choice::new(padding + 155, y, 150, field_height, None);
        category_l2.add_choice("All Level 2");
        category_l2.set_value(0);
        category_l2.deactivate();
        
        let mut category_l3 = Choice::new(padding + 310, y, 150, field_height, None);
        category_l3.add_choice("All Level 3");
        category_l3.set_value(0);
        category_l3.deactivate();
        
        y += field_height + spacing;
        
        // 年份范围
        let _year_label = Frame::new(padding, y, 100, label_height, "Year range:");
        y += label_height + 5;
        
        let mut year_from = IntInput::new(padding, y, 100, field_height, None);
        year_from.set_color(Color::White);
        
        let _to_label = Frame::new(padding + 110, y + 5, 30, 20, "to");
        
        let mut year_to = IntInput::new(padding + 140, y, 100, field_height, None);
        year_to.set_color(Color::White);
        
        y += field_height + spacing;
        
        // 出版商
        let _publisher_label = Frame::new(padding, y, 100, label_height, "Publisher:");
        y += label_height + 5;
        let mut publisher_input = Input::new(padding, y, 470, field_height, None);
        publisher_input.set_color(Color::White);
        y += field_height + spacing;
        
        // 摘要搜索
        let _summary_label = Frame::new(padding, y, 100, label_height, "Summary contains:");
        y += label_height + 5;
        let mut summary_input = TextEditor::new(padding, y, 470, 80, None);
        summary_input.set_color(Color::White);
        let summary_buffer = TextBuffer::default();
        summary_input.set_buffer(Some(summary_buffer.clone()));
        y += 80 + spacing;
        
        // 按钮区域
        let mut button_flex = Flex::new(padding, y, 470, 40, None);
        button_flex.set_type(FlexType::Row);
        button_flex.set_spacing(10);
        
        let mut search_btn = Button::new(0, 0, 0, 0, "Search");
        search_btn.set_color(Color::from_rgb(0, 123, 255));
        search_btn.set_label_color(Color::White);
        
        let mut clear_btn = Button::new(0, 0, 0, 0, "Clear");
        clear_btn.set_color(Color::from_rgb(108, 117, 125));
        clear_btn.set_label_color(Color::White);
        
        let mut cancel_btn = Button::new(0, 0, 0, 0, "Cancel");
        cancel_btn.set_color(Color::from_rgb(220, 53, 69));
        cancel_btn.set_label_color(Color::White);
        
        button_flex.end();
        window.end();
        
        // 设置按钮回调
        let sender_clone = event_sender.clone();
        search_btn.set_callback(move |_| {
            let _ = sender_clone.send(AppEvent::RefreshView); // 临时实现
        });
        
        let sender_clone = event_sender.clone();
        cancel_btn.set_callback(move |_| {
            // 关闭对话框
        });
        
        // 分类级联选择
        category_l1.set_callback(move |choice| {
            let selected = choice.value();
            if selected > 0 {
                // TODO: 加载二级分类
            }
        });
        
        Self {
            window,
            title_input,
            author_input,
            tag_input,
            category_l1,
            category_l2,
            category_l3,
            year_from,
            year_to,
            publisher_input,
            summary_input,
            summary_buffer,
            search_btn,
            clear_btn,
            cancel_btn,
            event_sender,
        }
    }
    
    pub fn show(&mut self) {
        self.window.show();
    }
    
    pub fn hide(&mut self) {
        self.window.hide();
    }
    
    pub async fn populate_dropdowns(&mut self, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        // 获取所有分类数据
        let search_result = tagbox_core::search_files_advanced("", None, config).await?;
        
        let mut level1_categories = std::collections::HashSet::new();
        
        for file in &search_result.entries {
            level1_categories.insert(file.category1.clone());
        }
        
        // 填充一级分类下拉框
        self.category_l1.clear();
        self.category_l1.add_choice("All Categories");
        
        let mut sorted_categories: Vec<_> = level1_categories.into_iter().collect();
        sorted_categories.sort();
        
        for category in sorted_categories {
            self.category_l1.add_choice(&category);
        }
        
        Ok(())
    }
    
    pub fn build_search_options(&self) -> SearchOptions {
        SearchOptions {
            limit: 100,
            offset: 0,
            sort_by: None,
            sort_direction: None,
            include_deleted: false,
        }
    }
    
    pub fn build_query_string(&self) -> String {
        let mut query_parts = Vec::new();
        
        // 标题搜索
        let title = self.title_input.value();
        if !title.trim().is_empty() {
            query_parts.push(format!("title:{}", title));
        }
        
        // 作者搜索
        let author = self.author_input.value();
        if !author.trim().is_empty() {
            query_parts.push(format!("author:{}", author));
        }
        
        // 标签搜索
        let tags = self.tag_input.value();
        if !tags.trim().is_empty() {
            for tag in tags.split(',') {
                let tag = tag.trim();
                if !tag.is_empty() {
                    query_parts.push(format!("tag:{}", tag));
                }
            }
        }
        
        // 分类搜索
        if self.category_l1.value() > 0 {
            if let Some(category) = self.category_l1.choice() {
                query_parts.push(format!("category1:{}", category));
            }
        }
        
        // 年份范围
        let year_from = self.year_from.value();
        let year_to = self.year_to.value();
        
        if !year_from.is_empty() {
            if let Ok(year) = year_from.parse::<i32>() {
                query_parts.push(format!("year:>={}", year));
            }
        }
        
        if !year_to.is_empty() {
            if let Ok(year) = year_to.parse::<i32>() {
                query_parts.push(format!("year:<={}", year));
            }
        }
        
        // 出版商搜索
        let publisher = self.publisher_input.value();
        if !publisher.trim().is_empty() {
            query_parts.push(format!("publisher:{}", publisher));
        }
        
        // 摘要搜索
        let summary = self.summary_buffer.text();
        if !summary.trim().is_empty() {
            query_parts.push(format!("summary:{}", summary));
        }
        
        if query_parts.is_empty() {
            "*".to_string()
        } else {
            query_parts.join(" ")
        }
    }
    
    pub fn clear_form(&mut self) {
        self.title_input.set_value("");
        self.author_input.set_value("");
        self.tag_input.set_value("");
        self.category_l1.set_value(0);
        self.category_l2.set_value(0);
        self.category_l3.set_value(0);
        self.year_from.set_value("");
        self.year_to.set_value("");
        self.publisher_input.set_value("");
        self.summary_buffer.set_text("");
    }
}