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
    dialog::NativeFileChooser,
};
use std::sync::mpsc::Sender;
use std::path::PathBuf;
use tagbox_core::{config::AppConfig, types::ImportMetadata};
use crate::state::AppEvent;

pub struct ImportDialog {
    window: Window,
    
    // 文件选择区域
    file_path_input: Input,
    browse_btn: Button,
    url_input: Input,
    download_btn: Button,
    
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
    extract_meta_btn: Button,
    import_move_btn: Button,
    import_keep_btn: Button,
    cancel_btn: Button,
    
    // 状态
    selected_file: Option<PathBuf>,
    event_sender: Sender<AppEvent>,
}

impl ImportDialog {
    pub fn new(event_sender: Sender<AppEvent>) -> Self {
        let mut window = Window::new(150, 150, 600, 700, "Import File");
        window.make_modal(true);
        window.set_color(Color::from_rgb(248, 249, 250));
        
        let padding = 15;
        let field_height = 30;
        let label_height = 20;
        let spacing = 10;
        let mut y = padding;
        
        // 文件选择区域
        let _file_section = Frame::new(padding, y, 570, label_height, "File Selection");
        y += label_height + 10;
        
        // 本地文件选择
        let _file_label = Frame::new(padding, y, 100, label_height, "File path:");
        y += label_height + 5;
        
        let mut file_path_input = Input::new(padding, y, 450, field_height, None);
        file_path_input.set_color(Color::White);
        file_path_input.set_readonly(true);
        
        let mut browse_btn = Button::new(padding + 460, y, 110, field_height, "Browse...");
        browse_btn.set_color(Color::from_rgb(108, 117, 125));
        browse_btn.set_label_color(Color::White);
        
        y += field_height + spacing;
        
        // URL 下载（可选）
        let _url_label = Frame::new(padding, y, 100, label_height, "Or URL:");
        y += label_height + 5;
        
        let mut url_input = Input::new(padding, y, 450, field_height, None);
        url_input.set_color(Color::White);
        
        let mut download_btn = Button::new(padding + 460, y, 110, field_height, "Download");
        download_btn.set_color(Color::from_rgb(23, 162, 184));
        download_btn.set_label_color(Color::White);
        download_btn.deactivate();
        
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
        let mut summary_editor = TextEditor::new(padding, y, 570, 100, None);
        summary_editor.set_color(Color::White);
        let summary_buffer = TextBuffer::default();
        summary_editor.set_buffer(Some(summary_buffer.clone()));
        y += 100 + spacing + 10;
        
        // 操作按钮区域
        let mut button_flex = Flex::new(padding, y, 570, 40, None);
        button_flex.set_type(FlexType::Row);
        button_flex.set_spacing(10);
        
        let mut extract_meta_btn = Button::new(0, 0, 0, 0, "Extract Metadata");
        extract_meta_btn.set_color(Color::from_rgb(255, 193, 7));
        extract_meta_btn.set_label_color(Color::Black);
        
        let mut import_move_btn = Button::new(0, 0, 0, 0, "Import & Move");
        import_move_btn.set_color(Color::from_rgb(40, 167, 69));
        import_move_btn.set_label_color(Color::White);
        
        let mut import_keep_btn = Button::new(0, 0, 0, 0, "Import & Keep");
        import_keep_btn.set_color(Color::from_rgb(0, 123, 255));
        import_keep_btn.set_label_color(Color::White);
        
        let mut cancel_btn = Button::new(0, 0, 0, 0, "Cancel");
        cancel_btn.set_color(Color::from_rgb(220, 53, 69));
        cancel_btn.set_label_color(Color::White);
        
        button_flex.end();
        window.end();
        
        // 设置回调
        let mut path_input_clone = file_path_input.clone();
        browse_btn.set_callback(move |_| {
            let mut dialog = NativeFileChooser::new(fltk::dialog::NativeFileChooserType::BrowseFile);
            dialog.set_title("Select file to import");
            dialog.set_filter("All Files\t*");
            dialog.show();
            
            let path = dialog.filename();
            if !path.to_string_lossy().is_empty() {
                path_input_clone.set_value(&path.to_string_lossy());
            }
        });
        
        Self {
            window,
            file_path_input,
            browse_btn,
            url_input,
            download_btn,
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
            extract_meta_btn,
            import_move_btn,
            import_keep_btn,
            cancel_btn,
            selected_file: None,
            event_sender,
        }
    }
    
    pub fn show(&mut self) {
        self.window.show();
    }
    
    pub fn hide(&mut self) {
        self.window.hide();
    }
    
    pub fn set_file(&mut self, path: PathBuf) {
        self.file_path_input.set_value(&path.to_string_lossy());
        self.selected_file = Some(path);
        
        // 启用提取元数据按钮
        self.extract_meta_btn.activate();
        self.import_move_btn.activate();
        self.import_keep_btn.activate();
    }
    
    pub async fn extract_metadata(&mut self, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = &self.selected_file {
            let metadata = tagbox_core::extract_metainfo(path, config).await?;
            self.populate_form(metadata);
        }
        Ok(())
    }
    
    pub fn populate_form(&mut self, metadata: ImportMetadata) {
        self.title_input.set_value(&metadata.title);
        self.authors_input.set_value(&metadata.authors.join(", "));
        
        if let Some(year) = metadata.year {
            self.year_input.set_value(&year.to_string());
        }
        
        if let Some(publisher) = metadata.publisher {
            self.publisher_input.set_value(&publisher);
        }
        
        if let Some(source) = metadata.source {
            self.source_input.set_value(&source);
        }
        
        self.tags_input.set_value(&metadata.tags.join(", "));
        
        if let Some(summary) = metadata.summary {
            self.summary_buffer.set_text(&summary);
        }
        
        // 设置分类
        // TODO: 实现分类的设置逻辑
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
        
        ImportMetadata {
            title: self.title_input.value(),
            authors,
            year,
            publisher,
            source,
            category1: "Default".to_string(), // TODO: 从分类选择器获取
            category2: None,
            category3: None,
            tags,
            summary,
            full_text: None,
            additional_info: std::collections::HashMap::new(),
            file_metadata: None,
            type_metadata: None,
        }
    }
    
    pub async fn import_file(&self, config: &AppConfig, move_file: bool) -> Result<tagbox_core::types::FileEntry, Box<dyn std::error::Error>> {
        if let Some(path) = &self.selected_file {
            let metadata = self.collect_form_data();
            
            // 调用 tagbox-core 的导入功能
            let file_entry = tagbox_core::import_file(path, metadata, config).await?;
            
            if move_file {
                // TODO: 实现文件移动逻辑
                // 这里需要根据配置将文件移动到目标位置
            }
            
            Ok(file_entry)
        } else {
            Err("No file selected".into())
        }
    }
    
    pub fn clear_form(&mut self) {
        self.file_path_input.set_value("");
        self.url_input.set_value("");
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
        self.selected_file = None;
        
        // 禁用按钮
        self.extract_meta_btn.deactivate();
        self.import_move_btn.deactivate();
        self.import_keep_btn.deactivate();
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
}