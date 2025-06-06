use fltk::{
    prelude::*,
    group::{Group, Flex, FlexType},
    text::{TextDisplay, TextBuffer},
    output::Output,
    button::Button,
    browser::Browser,
    enums::Color,
    frame::Frame,
};
use std::sync::mpsc::Sender;
use tagbox_core::{config::AppConfig, types::FileEntry};
use crate::state::AppEvent;
use crate::utils::{copy_to_clipboard, open_folder};

pub struct FilePreview {
    container: Group,
    
    // 标题区域
    title_frame: Frame,
    title_output: Output,
    
    // 基本信息区域
    info_flex: Flex,
    path_output: Output,
    authors_output: Output,
    year_output: Output,
    publisher_output: Output,
    
    // 标签区域
    tags_flex: Flex,
    tags_display: TextDisplay,
    tags_buffer: TextBuffer,
    
    // 摘要区域
    summary_display: TextDisplay,
    summary_buffer: TextBuffer,
    
    // 关联文件区域
    links_browser: Browser,
    
    // 操作按钮区域
    buttons_flex: Flex,
    open_btn: Button,
    edit_btn: Button,
    copy_path_btn: Button,
    cd_btn: Button,
    
    // 状态
    current_file: Option<FileEntry>,
    event_sender: Sender<AppEvent>,
}

impl FilePreview {
    pub fn new(
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        event_sender: Sender<AppEvent>
    ) -> Self {
        let mut container = Group::new(x, y, w, h, None);
        container.set_color(Color::from_rgb(248, 249, 250));
        
        let padding = 10;
        let content_x = x + padding;
        let content_w = w - 2 * padding;
        let mut current_y = y + padding;
        
        // 标题区域
        let mut title_frame = Frame::new(content_x, current_y, content_w, 25, "File Details");
        title_frame.set_label_size(16);
        title_frame.set_label_color(Color::from_rgb(33, 37, 41));
        current_y += 35;
        
        let mut title_output = Output::new(content_x, current_y, content_w, 30, None);
        title_output.set_color(Color::White);
        title_output.set_text_size(14);
        current_y += 40;
        
        // 基本信息区域
        let mut info_flex = Flex::new(content_x, current_y, content_w, 120, None);
        info_flex.set_type(FlexType::Column);
        info_flex.set_spacing(5);
        
        let mut path_output = Output::new(0, 0, 0, 25, "Path:");
        path_output.set_color(Color::from_rgb(248, 249, 250));
        path_output.set_text_size(10);
        
        let mut authors_output = Output::new(0, 0, 0, 25, "Authors:");
        authors_output.set_color(Color::from_rgb(248, 249, 250));
        
        let mut year_output = Output::new(0, 0, 0, 25, "Year:");
        year_output.set_color(Color::from_rgb(248, 249, 250));
        
        let mut publisher_output = Output::new(0, 0, 0, 25, "Publisher:");
        publisher_output.set_color(Color::from_rgb(248, 249, 250));
        
        info_flex.end();
        current_y += 130;
        
        // 标签区域
        let mut tags_flex = Flex::new(content_x, current_y, content_w, 60, None);
        tags_flex.set_type(FlexType::Column);
        
        let _tags_label = Frame::new(0, 0, 0, 20, "Tags:");
        
        let mut tags_display = TextDisplay::new(0, 0, 0, 40, None);
        tags_display.set_color(Color::White);
        let tags_buffer = TextBuffer::default();
        tags_display.set_buffer(Some(tags_buffer.clone()));
        
        tags_flex.end();
        current_y += 70;
        
        // 摘要区域
        let _summary_label = Frame::new(content_x, current_y, content_w, 20, "Summary:");
        current_y += 25;
        
        let mut summary_display = TextDisplay::new(content_x, current_y, content_w, 100, None);
        summary_display.set_color(Color::White);
        let summary_buffer = TextBuffer::default();
        summary_display.set_buffer(Some(summary_buffer.clone()));
        current_y += 110;
        
        // 关联文件区域
        let _links_label = Frame::new(content_x, current_y, content_w, 20, "Related Files:");
        current_y += 25;
        
        let mut links_browser = Browser::new(content_x, current_y, content_w, 80, None);
        links_browser.set_color(Color::White);
        current_y += 90;
        
        // 操作按钮区域
        let remaining_height = (y + h) - current_y - padding;
        let mut buttons_flex = Flex::new(content_x, current_y, content_w, remaining_height, None);
        buttons_flex.set_type(FlexType::Row);
        buttons_flex.set_spacing(10);
        
        let mut open_btn = Button::new(0, 0, 0, 30, "Open File");
        open_btn.set_color(Color::from_rgb(40, 167, 69));
        open_btn.set_label_color(Color::White);
        
        let mut edit_btn = Button::new(0, 0, 0, 30, "Edit");
        edit_btn.set_color(Color::from_rgb(0, 123, 255));
        edit_btn.set_label_color(Color::White);
        
        let mut copy_path_btn = Button::new(0, 0, 0, 30, "Copy Path");
        copy_path_btn.set_color(Color::from_rgb(108, 117, 125));
        copy_path_btn.set_label_color(Color::White);
        
        let mut cd_btn = Button::new(0, 0, 0, 30, "Open Folder");
        cd_btn.set_color(Color::from_rgb(23, 162, 184));
        cd_btn.set_label_color(Color::White);
        
        buttons_flex.end();
        container.end();
        
        // 设置按钮回调
        let sender_clone = event_sender.clone();
        open_btn.set_callback(move |_| {
            let _ = sender_clone.send(AppEvent::FileOpen("current".to_string()));
        });
        
        let sender_clone = event_sender.clone();
        edit_btn.set_callback(move |_| {
            let _ = sender_clone.send(AppEvent::FileEdit("current".to_string()));
        });
        
        copy_path_btn.set_callback(move |_| {
            // 复制路径功能在 display_file 中实现
        });
        
        cd_btn.set_callback(move |_| {
            // 打开文件夹功能在 display_file 中实现
        });
        
        Self {
            container,
            title_frame,
            title_output,
            info_flex,
            path_output,
            authors_output,
            year_output,
            publisher_output,
            tags_flex,
            tags_display,
            tags_buffer,
            summary_display,
            summary_buffer,
            links_browser,
            buttons_flex,
            open_btn,
            edit_btn,
            copy_path_btn,
            cd_btn,
            current_file: None,
            event_sender,
        }
    }
    
    pub async fn display_file(&mut self, file_id: &str, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        // 从 tagbox-core 获取文件详情
        let file = tagbox_core::get_file(file_id, config).await?;
        
        // 更新标题
        let title = if file.title.is_empty() {
            &file.original_filename
        } else {
            &file.title
        };
        self.title_output.set_value(title);
        
        // 更新基本信息
        self.path_output.set_value(&file.path.to_string_lossy());
        self.authors_output.set_value(&file.authors.join(", "));
        
        if let Some(year) = file.year {
            self.year_output.set_value(&year.to_string());
        } else {
            self.year_output.set_value("N/A");
        }
        
        if let Some(publisher) = &file.publisher {
            self.publisher_output.set_value(publisher);
        } else {
            self.publisher_output.set_value("N/A");
        }
        
        // 更新标签
        let tags_text = if file.tags.is_empty() {
            "No tags".to_string()
        } else {
            file.tags.join(", ")
        };
        self.tags_buffer.set_text(&tags_text);
        
        // 更新摘要
        let summary_text = file.summary.as_deref().unwrap_or("No summary available");
        self.summary_buffer.set_text(summary_text);
        
        // 获取并显示关联文件
        self.links_browser.clear();
        // TODO: 实现关联文件获取
        // let links = tagbox_core::LinkManager::get_links(&file.id, config).await?;
        // for link in links {
        //     self.links_browser.add(&format!("{} -> {}", link.relation, link.target_title));
        // }
        
        // 更新按钮回调以包含当前文件信息
        let file_path = file.path.clone();
        let file_path_copy = file_path.clone();
        
        self.copy_path_btn.set_callback(move |_| {
            if let Err(e) = copy_to_clipboard(&file_path.to_string_lossy()) {
                eprintln!("Failed to copy to clipboard: {}", e);
            }
        });
        
        self.cd_btn.set_callback(move |_| {
            if let Err(e) = open_folder(&file_path_copy) {
                eprintln!("Failed to open folder: {}", e);
            }
        });
        
        // 保存当前文件
        self.current_file = Some(file);
        
        // 重绘容器
        self.container.redraw();
        
        Ok(())
    }
    
    pub fn clear(&mut self) {
        self.title_output.set_value("No file selected");
        self.path_output.set_value("");
        self.authors_output.set_value("");
        self.year_output.set_value("");
        self.publisher_output.set_value("");
        self.tags_buffer.set_text("");
        self.summary_buffer.set_text("");
        self.links_browser.clear();
        self.current_file = None;
        self.container.redraw();
    }
    
    pub fn get_current_file(&self) -> Option<&FileEntry> {
        self.current_file.as_ref()
    }
    
    pub fn refresh(&mut self) {
        if let Some(file) = &self.current_file {
            let file_id = file.id.clone();
            // 异步刷新需要通过事件系统
            let _ = self.event_sender.send(AppEvent::FileSelected(file_id));
        }
    }
    
    // 设置加载状态
    pub fn set_loading(&mut self, loading: bool) {
        if loading {
            self.title_output.set_value("Loading...");
            self.path_output.set_value("");
            self.authors_output.set_value("");
            self.year_output.set_value("");
            self.publisher_output.set_value("");
            self.tags_buffer.set_text("");
            self.summary_buffer.set_text("");
            self.links_browser.clear();
        }
        
        // 禁用/启用按钮
        if loading {
            self.open_btn.deactivate();
            self.edit_btn.deactivate();
            self.copy_path_btn.deactivate();
            self.cd_btn.deactivate();
        } else {
            self.open_btn.activate();
            self.edit_btn.activate();
            self.copy_path_btn.activate();
            self.cd_btn.activate();
        }
        
        self.container.redraw();
    }
    
    // 获取容器的引用（用于主窗口布局）
    pub fn widget(&mut self) -> &mut Group {
        &mut self.container
    }
}