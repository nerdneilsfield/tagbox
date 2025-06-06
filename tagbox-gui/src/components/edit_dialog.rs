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
use tagbox_core::{config::AppConfig, types::{FileEntry, ImportMetadata}};
use crate::state::AppEvent;

pub struct EditDialog {
    window: Window,
    
    // 文件信息
    file_id: Option<String>,
    path_output: Input,
    
    // 元数据表单
    title_input: Input,
    authors_input: Input,
    year_input: IntInput,
    publisher_input: Input,
    source_input: Input,
    tags_input: Input,
    summary_editor: TextEditor,
    summary_buffer: TextBuffer,
    
    // 分类选择
    category_l1: Choice,
    category_l2: Choice,
    category_l3: Choice,
    
    // 操作按钮
    save_btn: Button,
    delete_btn: Button,
    cancel_btn: Button,
    
    event_sender: Sender<AppEvent>,
}

impl EditDialog {
    pub fn new(event_sender: Sender<AppEvent>) -> Self {
        let mut window = Window::new(150, 150, 600, 650, "Edit File");
        window.make_modal(true);
        window.set_color(Color::from_rgb(248, 249, 250));
        
        let padding = 15;
        let field_height = 30;
        let label_height = 20;
        let spacing = 10;
        let mut y = padding;
        
        // 文件路径（只读）
        let _path_section = Frame::new(padding, y, 570, label_height, "File Information");
        y += label_height + 10;
        
        let _path_label = Frame::new(padding, y, 100, label_height, "File path:");
        y += label_height + 5;
        
        let mut path_output = Input::new(padding, y, 570, field_height, None);
        path_output.set_color(Color::from_rgb(248, 249, 250));
        path_output.set_readonly(true);
        
        y += field_height + spacing + 10;
        
        // 元数据表单
        let _meta_section = Frame::new(padding, y, 570, label_height, "Metadata");
        y += label_height + 10;
        
        // 标题
        let _title_label = Frame::new(padding, y, 100, label_height, "Title:");
        y += label_height + 5;
        let mut title_input = Input::new(padding, y, 570, field_height, None);
        title_input.set_color(Color::White);
        y += field_height + spacing;
        
        // 作者
        let _authors_label = Frame::new(padding, y, 100, label_height, "Authors (comma separated):");
        y += label_height + 5;
        let mut authors_input = Input::new(padding, y, 570, field_height, None);
        authors_input.set_color(Color::White);
        y += field_height + spacing;
        
        // 年份和出版商（同一行）
        let _year_label = Frame::new(padding, y, 50, label_height, "Year:");
        let _publisher_label = Frame::new(padding + 200, y, 80, label_height, "Publisher:");
        y += label_height + 5;
        
        let mut year_input = IntInput::new(padding, y, 100, field_height, None);
        year_input.set_color(Color::White);
        
        let mut publisher_input = Input::new(padding + 200, y, 370, field_height, None);
        publisher_input.set_color(Color::White);
        
        y += field_height + spacing;
        
        // 来源
        let _source_label = Frame::new(padding, y, 100, label_height, "Source:");
        y += label_height + 5;
        let mut source_input = Input::new(padding, y, 570, field_height, None);
        source_input.set_color(Color::White);
        y += field_height + spacing;
        
        // 标签
        let _tags_label = Frame::new(padding, y, 100, label_height, "Tags (comma separated):");
        y += label_height + 5;
        let mut tags_input = Input::new(padding, y, 570, field_height, None);
        tags_input.set_color(Color::White);
        y += field_height + spacing;
        
        // 分类选择
        let _category_label = Frame::new(padding, y, 100, label_height, "Category:");
        y += label_height + 5;
        
        let mut category_l1 = Choice::new(padding, y, 180, field_height, None);
        category_l1.add_choice("Select Category 1");
        category_l1.set_value(0);
        
        let mut category_l2 = Choice::new(padding + 190, y, 180, field_height, None);
        category_l2.add_choice("Select Category 2");
        category_l2.set_value(0);
        category_l2.deactivate();
        
        let mut category_l3 = Choice::new(padding + 380, y, 180, field_height, None);
        category_l3.add_choice("Select Category 3");
        category_l3.set_value(0);
        category_l3.deactivate();
        
        y += field_height + spacing;
        
        // 摘要
        let _summary_label = Frame::new(padding, y, 100, label_height, "Summary:");
        y += label_height + 5;
        let mut summary_editor = TextEditor::new(padding, y, 570, 80, None);
        summary_editor.set_color(Color::White);
        let summary_buffer = TextBuffer::default();
        summary_editor.set_buffer(Some(summary_buffer.clone()));
        y += 80 + spacing + 10;
        
        // 操作按钮区域
        let mut button_flex = Flex::new(padding, y, 570, 40, None);
        button_flex.set_type(FlexType::Row);
        button_flex.set_spacing(10);
        
        let mut save_btn = Button::new(0, 0, 0, 0, "Save Changes");
        save_btn.set_color(Color::from_rgb(40, 167, 69));
        save_btn.set_label_color(Color::White);
        
        let mut delete_btn = Button::new(0, 0, 0, 0, "Delete File");
        delete_btn.set_color(Color::from_rgb(220, 53, 69));
        delete_btn.set_label_color(Color::White);
        
        let mut cancel_btn = Button::new(0, 0, 0, 0, "Cancel");
        cancel_btn.set_color(Color::from_rgb(108, 117, 125));
        cancel_btn.set_label_color(Color::White);
        
        button_flex.end();
        window.end();
        
        Self {
            window,
            file_id: None,
            path_output,
            title_input,
            authors_input,
            year_input,
            publisher_input,
            source_input,
            tags_input,
            summary_editor,
            summary_buffer,
            category_l1,
            category_l2,
            category_l3,
            save_btn,
            delete_btn,
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
    
    pub async fn load_file(&mut self, file_id: &str, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        // 从 tagbox-core 获取文件信息
        let file = tagbox_core::get_file(file_id, config).await?;
        
        self.file_id = Some(file_id.to_string());
        self.populate_form(&file);
        
        // 加载分类选项
        self.populate_categories(config).await?;
        
        Ok(())
    }
    
    pub fn populate_form(&mut self, file: &FileEntry) {
        // 设置文件路径
        self.path_output.set_value(&file.path.to_string_lossy());
        
        // 设置元数据
        self.title_input.set_value(&file.title);
        self.authors_input.set_value(&file.authors.join(", "));
        
        if let Some(year) = file.year {
            self.year_input.set_value(&year.to_string());
        } else {
            self.year_input.set_value("");
        }
        
        if let Some(publisher) = &file.publisher {
            self.publisher_input.set_value(publisher);
        } else {
            self.publisher_input.set_value("");
        }
        
        if let Some(source) = &file.source {
            self.source_input.set_value(source);
        } else {
            self.source_input.set_value("");
        }
        
        self.tags_input.set_value(&file.tags.join(", "));
        
        if let Some(summary) = &file.summary {
            self.summary_buffer.set_text(summary);
        } else {
            self.summary_buffer.set_text("");
        }
        
        // 设置分类（简化版）
        // TODO: 实现分类选择器的设置逻辑
        self.set_category_selections(&file.category1, file.category2.as_deref(), file.category3.as_deref());
    }
    
    pub fn set_category_selections(&mut self, cat1: &str, cat2: Option<&str>, cat3: Option<&str>) {
        // 查找并设置一级分类
        // 查找并设置一级分类（简化版本）
        for i in 0..self.category_l1.size() {
            if let Some(choice_item) = self.category_l1.at(i) {
                if choice_item.label() == Some(cat1.to_string()) {
                    self.category_l1.set_value(i);
                    break;
                }
            }
        }
        
        // TODO: 实现二级和三级分类的设置
        if let Some(_cat2) = cat2 {
            self.category_l2.activate();
            // 设置二级分类...
        }
        
        if let Some(_cat3) = cat3 {
            self.category_l3.activate();
            // 设置三级分类...
        }
    }
    
    pub fn collect_form_data(&self) -> ImportMetadata {
        let authors: Vec<String> = self.authors_input.value()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        let tags: Vec<String> = self.tags_input.value()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        
        let year = if self.year_input.value().is_empty() {
            None
        } else {
            self.year_input.value().parse().ok()
        };
        
        let publisher = if self.publisher_input.value().trim().is_empty() {
            None
        } else {
            Some(self.publisher_input.value())
        };
        
        let source = if self.source_input.value().trim().is_empty() {
            None
        } else {
            Some(self.source_input.value())
        };
        
        let summary = if self.summary_buffer.text().trim().is_empty() {
            None
        } else {
            Some(self.summary_buffer.text())
        };
        
        // 获取选择的分类
        let category1 = if self.category_l1.value() > 0 {
            self.category_l1.choice().unwrap_or_else(|| "Default".to_string())
        } else {
            "Default".to_string()
        };
        
        let category2 = if self.category_l2.value() > 0 {
            self.category_l2.choice()
        } else {
            None
        };
        
        let category3 = if self.category_l3.value() > 0 {
            self.category_l3.choice()
        } else {
            None
        };
        
        ImportMetadata {
            title: self.title_input.value(),
            authors,
            year,
            publisher,
            source,
            category1,
            category2,
            category3,
            tags,
            summary,
            full_text: None,
            additional_info: std::collections::HashMap::new(),
            file_metadata: None,
            type_metadata: None,
        }
    }
    
    pub async fn save_changes(&self, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(file_id) = &self.file_id {
            let metadata = self.collect_form_data();
            
            // 调用 tagbox-core 的更新功能
            // TODO: 调用 tagbox-core 的更新功能 - API 待实现
            // tagbox_core::update_file_metadata(file_id, metadata, config).await?;
            
            Ok(())
        } else {
            Err("No file loaded for editing".into())
        }
    }
    
    pub async fn delete_file(&self, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(file_id) = &self.file_id {
            // 调用 tagbox-core 的删除功能（软删除）
            // TODO: 调用 tagbox-core 的删除功能（软删除） - API 待实现
            // tagbox_core::soft_delete_file(file_id, config).await?;
            
            Ok(())
        } else {
            Err("No file loaded for deletion".into())
        }
    }
    
    pub fn clear_form(&mut self) {
        self.file_id = None;
        self.path_output.set_value("");
        self.title_input.set_value("");
        self.authors_input.set_value("");
        self.year_input.set_value("");
        self.publisher_input.set_value("");
        self.source_input.set_value("");
        self.tags_input.set_value("");
        self.summary_buffer.set_text("");
        self.category_l1.set_value(0);
        self.category_l2.set_value(0);
        self.category_l3.set_value(0);
        self.category_l2.deactivate();
        self.category_l3.deactivate();
    }
    
    pub async fn populate_categories(&mut self, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        // 获取所有分类数据并填充下拉框
        let search_result = tagbox_core::search_files_advanced("", None, config).await?;
        
        let mut level1_categories = std::collections::HashSet::new();
        
        for file in &search_result.entries {
            level1_categories.insert(file.category1.clone());
        }
        
        // 填充一级分类下拉框
        self.category_l1.clear();
        self.category_l1.add_choice("Select Category 1");
        
        let mut sorted_categories: Vec<_> = level1_categories.into_iter().collect();
        sorted_categories.sort();
        
        for category in sorted_categories {
            self.category_l1.add_choice(&category);
        }
        
        Ok(())
    }
    
    pub fn set_callbacks(&mut self) {
        // 保存按钮回调
        let sender_clone = self.event_sender.clone();
        self.save_btn.set_callback(move |_| {
            let _ = sender_clone.send(AppEvent::SaveFile);
        });
        
        // 删除按钮回调
        let sender_clone = self.event_sender.clone();
        self.delete_btn.set_callback(move |_| {
            // 显示确认对话框
            let choice = fltk::dialog::choice2_default(
                "Are you sure you want to delete this file?",
                "Yes",
                "No",
                ""
            );
            
            if choice == Some(0) {
                let _ = sender_clone.send(AppEvent::DeleteFile);
            }
        });
        
        // 取消按钮回调
        let sender_clone = self.event_sender.clone();
        self.cancel_btn.set_callback(move |_| {
            let _ = sender_clone.send(AppEvent::CancelEdit);
        });
        
        // 分类级联选择
        self.category_l1.set_callback(move |choice| {
            let selected = choice.value();
            if selected > 0 {
                // TODO: 加载二级分类
            }
        });
    }
    
    pub fn validate_form(&self) -> Result<(), String> {
        // 基本验证
        if self.title_input.value().trim().is_empty() {
            return Err("Title is required".to_string());
        }
        
        // 年份验证
        if !self.year_input.value().is_empty() {
            if let Err(_) = self.year_input.value().parse::<i32>() {
                return Err("Invalid year format".to_string());
            }
        }
        
        Ok(())
    }
    
    pub fn set_loading(&mut self, loading: bool) {
        if loading {
            self.save_btn.deactivate();
            self.delete_btn.deactivate();
            self.title_input.deactivate();
            self.authors_input.deactivate();
            self.year_input.deactivate();
            self.publisher_input.deactivate();
            self.source_input.deactivate();
            self.tags_input.deactivate();
            self.summary_editor.deactivate();
        } else {
            self.save_btn.activate();
            self.delete_btn.activate();
            self.title_input.activate();
            self.authors_input.activate();
            self.year_input.activate();
            self.publisher_input.activate();
            self.source_input.activate();
            self.tags_input.activate();
            self.summary_editor.activate();
        }
    }
}