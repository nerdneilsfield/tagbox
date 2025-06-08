use fltk::{
    prelude::*,
    window::Window,
    group::{Flex, FlexType, Group, Tabs},
    button::Button,
    tree::Tree,
    enums::{Color, FrameType},
    frame::Frame,
    text::{TextDisplay, TextBuffer},
    output::Output,
};
use std::sync::mpsc::Sender;
use std::collections::HashMap;
use tagbox_core::config::AppConfig;
use crate::state::AppEvent;

pub struct StatisticsDialog {
    window: Window,
    
    // 标签页
    tabs: Tabs,
    
    // 概览标签页
    overview_group: Group,
    total_files_output: Output,
    total_size_output: Output,
    categories_count_output: Output,
    authors_count_output: Output,
    recent_imports_output: Output,
    
    // 分类统计标签页
    category_group: Group,
    category_tree: Tree,
    category_stats_text: TextDisplay,
    category_buffer: TextBuffer,
    
    // 文件类型统计标签页
    filetype_group: Group,
    filetype_stats_text: TextDisplay,
    filetype_buffer: TextBuffer,
    
    // 作者统计标签页
    author_group: Group,
    author_stats_text: TextDisplay,
    author_buffer: TextBuffer,
    
    // 时间趋势标签页
    timeline_group: Group,
    timeline_stats_text: TextDisplay,
    timeline_buffer: TextBuffer,
    
    // 控制按钮
    refresh_btn: Button,
    export_btn: Button,
    close_btn: Button,
    
    // 状态
    statistics: Option<FileStatistics>,
    event_sender: Sender<AppEvent>,
}

#[derive(Debug, Clone)]
pub struct FileStatistics {
    pub total_files: usize,
    pub total_size: u64,
    pub total_categories: usize,
    pub total_authors: usize,
    pub recent_imports: usize,
    pub category_distribution: HashMap<String, CategoryStats>,
    pub filetype_distribution: HashMap<String, FileTypeStats>,
    pub author_distribution: HashMap<String, AuthorStats>,
    pub import_timeline: Vec<ImportPeriod>,
}

#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub file_count: usize,
    pub total_size: u64,
    pub subcategories: Vec<String>,
    pub recent_files: usize,
}

#[derive(Debug, Clone)]
pub struct FileTypeStats {
    pub file_count: usize,
    pub total_size: u64,
    pub average_size: u64,
    pub largest_file: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuthorStats {
    pub file_count: usize,
    pub total_size: u64,
    pub categories: Vec<String>,
    pub years: Vec<i32>,
}

#[derive(Debug, Clone)]
pub struct ImportPeriod {
    pub period: String,
    pub file_count: usize,
    pub total_size: u64,
}

impl StatisticsDialog {
    pub fn new(event_sender: Sender<AppEvent>) -> Self {
        let mut window = Window::new(200, 200, 900, 700, "File Statistics & Reports");
        window.make_modal(true);
        window.set_color(Color::from_rgb(248, 249, 250));
        
        let padding = 15;
        let mut main_flex = Flex::new(padding, padding, 870, 670, None);
        main_flex.set_type(FlexType::Column);
        main_flex.set_spacing(15);
        
        // 标题
        let mut title_frame = Frame::new(0, 0, 870, 30, "TagBox File Statistics & Reports");
        title_frame.set_label_size(18);
        title_frame.set_label_color(Color::from_rgb(51, 51, 51));
        main_flex.fixed(&title_frame, 30);
        
        // 标签页容器
        let mut tabs = Tabs::new(0, 0, 870, 580, None);
        tabs.set_color(Color::White);
        
        // 1. 概览标签页
        let mut overview_group = Group::new(5, 25, 860, 550, "Overview");
        overview_group.set_color(Color::White);
        
        let mut overview_flex = Flex::new(10, 35, 850, 540, None);
        overview_flex.set_type(FlexType::Column);
        overview_flex.set_spacing(15);
        
        // 概览卡片网格
        let mut cards_flex = Flex::new(0, 0, 850, 200, None);
        cards_flex.set_type(FlexType::Row);
        cards_flex.set_spacing(15);
        
        // 总文件数卡片
        let mut total_files_group = Group::new(0, 0, 0, 200, None);
        total_files_group.set_frame(FrameType::BorderBox);
        total_files_group.set_color(Color::from_rgb(240, 248, 255));
        
        let mut files_label = Frame::new(10, 10, 0, 30, "Total Files");
        files_label.set_label_size(14);
        files_label.set_label_color(Color::from_rgb(51, 51, 51));
        
        let mut total_files_output = Output::new(10, 50, 0, 40, None);
        total_files_output.set_text_size(24);
        total_files_output.set_text_color(Color::from_rgb(0, 123, 255));
        total_files_output.set_color(Color::White);
        
        total_files_group.end();
        
        // 总大小卡片
        let mut total_size_group = Group::new(0, 0, 0, 200, None);
        total_size_group.set_frame(FrameType::BorderBox);
        total_size_group.set_color(Color::from_rgb(240, 255, 240));
        
        let mut size_label = Frame::new(10, 10, 0, 30, "Total Size");
        size_label.set_label_size(14);
        size_label.set_label_color(Color::from_rgb(51, 51, 51));
        
        let mut total_size_output = Output::new(10, 50, 0, 40, None);
        total_size_output.set_text_size(20);
        total_size_output.set_text_color(Color::from_rgb(40, 167, 69));
        total_size_output.set_color(Color::White);
        
        total_size_group.end();
        
        // 分类数卡片
        let mut categories_group = Group::new(0, 0, 0, 200, None);
        categories_group.set_frame(FrameType::BorderBox);
        categories_group.set_color(Color::from_rgb(255, 248, 240));
        
        let mut cat_label = Frame::new(10, 10, 0, 30, "Categories");
        cat_label.set_label_size(14);
        cat_label.set_label_color(Color::from_rgb(51, 51, 51));
        
        let mut categories_count_output = Output::new(10, 50, 0, 40, None);
        categories_count_output.set_text_size(24);
        categories_count_output.set_text_color(Color::from_rgb(255, 193, 7));
        categories_count_output.set_color(Color::White);
        
        categories_group.end();
        
        // 作者数卡片
        let mut authors_group = Group::new(0, 0, 0, 200, None);
        authors_group.set_frame(FrameType::BorderBox);
        authors_group.set_color(Color::from_rgb(248, 240, 255));
        
        let mut auth_label = Frame::new(10, 10, 0, 30, "Authors");
        auth_label.set_label_size(14);
        auth_label.set_label_color(Color::from_rgb(51, 51, 51));
        
        let mut authors_count_output = Output::new(10, 50, 0, 40, None);
        authors_count_output.set_text_size(24);
        authors_count_output.set_text_color(Color::from_rgb(138, 43, 226));
        authors_count_output.set_color(Color::White);
        
        authors_group.end();
        
        cards_flex.end();
        overview_flex.fixed(&cards_flex, 200);
        
        // 最近导入统计
        let mut recent_label = Frame::new(0, 0, 850, 25, "Recent Import Activity (Last 30 Days)");
        recent_label.set_label_size(16);
        recent_label.set_align(fltk::enums::Align::Left | fltk::enums::Align::Inside);
        overview_flex.fixed(&recent_label, 25);
        
        let mut recent_imports_output = Output::new(0, 0, 850, 100, None);
        recent_imports_output.set_color(Color::White);
        recent_imports_output.set_text_size(12);
        overview_flex.fixed(&recent_imports_output, 100);
        
        // 快速操作区域
        let mut quick_actions_label = Frame::new(0, 0, 850, 25, "Quick Actions");
        quick_actions_label.set_label_size(16);
        quick_actions_label.set_align(fltk::enums::Align::Left | fltk::enums::Align::Inside);
        overview_flex.fixed(&quick_actions_label, 25);
        
        let mut actions_flex = Flex::new(0, 0, 850, 40, None);
        actions_flex.set_type(FlexType::Row);
        actions_flex.set_spacing(10);
        
        let mut refresh_all_btn = Button::new(0, 0, 0, 40, "Refresh All Data");
        refresh_all_btn.set_color(Color::from_rgb(0, 123, 255));
        refresh_all_btn.set_label_color(Color::White);
        
        let mut backup_btn = Button::new(0, 0, 0, 40, "Backup Database");
        backup_btn.set_color(Color::from_rgb(40, 167, 69));
        backup_btn.set_label_color(Color::White);
        
        let mut cleanup_btn = Button::new(0, 0, 0, 40, "Cleanup Orphans");
        cleanup_btn.set_color(Color::from_rgb(255, 193, 7));
        cleanup_btn.set_label_color(Color::Black);
        
        actions_flex.end();
        overview_flex.fixed(&actions_flex, 40);
        
        overview_flex.end();
        overview_group.end();
        tabs.add(&overview_group);
        
        // 2. 分类统计标签页
        let mut category_group = Group::new(5, 25, 860, 550, "Categories");
        category_group.set_color(Color::White);
        
        let mut cat_flex = Flex::new(10, 35, 850, 540, None);
        cat_flex.set_type(FlexType::Row);
        cat_flex.set_spacing(15);
        
        // 分类树
        let mut category_tree = Tree::new(0, 0, 400, 540, None);
        category_tree.set_color(Color::White);
        cat_flex.fixed(&category_tree, 400);
        
        // 分类详细统计
        let mut category_buffer = TextBuffer::default();
        let mut category_stats_text = TextDisplay::new(0, 0, 435, 540, None);
        category_stats_text.set_buffer(category_buffer.clone());
        category_stats_text.set_color(Color::White);
        cat_flex.fixed(&category_stats_text, 435);
        
        cat_flex.end();
        category_group.end();
        tabs.add(&category_group);
        
        // 3. 文件类型统计标签页
        let mut filetype_group = Group::new(5, 25, 860, 550, "File Types");
        filetype_group.set_color(Color::White);
        
        let mut filetype_buffer = TextBuffer::default();
        let mut filetype_stats_text = TextDisplay::new(10, 35, 850, 540, None);
        filetype_stats_text.set_buffer(filetype_buffer.clone());
        filetype_stats_text.set_color(Color::White);
        
        filetype_group.end();
        tabs.add(&filetype_group);
        
        // 4. 作者统计标签页
        let mut author_group = Group::new(5, 25, 860, 550, "Authors");
        author_group.set_color(Color::White);
        
        let mut author_buffer = TextBuffer::default();
        let mut author_stats_text = TextDisplay::new(10, 35, 850, 540, None);
        author_stats_text.set_buffer(author_buffer.clone());
        author_stats_text.set_color(Color::White);
        
        author_group.end();
        tabs.add(&author_group);
        
        // 5. 时间趋势标签页
        let mut timeline_group = Group::new(5, 25, 860, 550, "Timeline");
        timeline_group.set_color(Color::White);
        
        let mut timeline_buffer = TextBuffer::default();
        let mut timeline_stats_text = TextDisplay::new(10, 35, 850, 540, None);
        timeline_stats_text.set_buffer(timeline_buffer.clone());
        timeline_stats_text.set_color(Color::White);
        
        timeline_group.end();
        tabs.add(&timeline_group);
        
        tabs.end();
        main_flex.fixed(&tabs, 580);
        
        // 底部控制按钮
        let mut buttons_flex = Flex::new(0, 0, 870, 40, None);
        buttons_flex.set_type(FlexType::Row);
        buttons_flex.set_spacing(10);
        
        // 左侧按钮
        let mut refresh_btn = Button::new(0, 0, 0, 40, "Refresh Statistics");
        refresh_btn.set_color(Color::from_rgb(0, 123, 255));
        refresh_btn.set_label_color(Color::White);
        
        let mut export_btn = Button::new(0, 0, 0, 40, "Export Report");
        export_btn.set_color(Color::from_rgb(40, 167, 69));
        export_btn.set_label_color(Color::White);
        
        // 间隔
        let spacer = Frame::new(0, 0, 0, 40, None);
        
        // 右侧关闭按钮
        let mut close_btn = Button::new(0, 0, 0, 40, "Close");
        close_btn.set_color(Color::from_rgb(108, 117, 125));
        close_btn.set_label_color(Color::White);
        buttons_flex.fixed(&close_btn, 100);
        
        buttons_flex.end();
        main_flex.fixed(&buttons_flex, 40);
        
        main_flex.end();
        window.end();
        
        Self {
            window,
            tabs,
            overview_group,
            total_files_output,
            total_size_output,
            categories_count_output,
            authors_count_output,
            recent_imports_output,
            category_group,
            category_tree,
            category_stats_text,
            category_buffer,
            filetype_group,
            filetype_stats_text,
            filetype_buffer,
            author_group,
            author_stats_text,
            author_buffer,
            timeline_group,
            timeline_stats_text,
            timeline_buffer,
            refresh_btn,
            export_btn,
            close_btn,
            statistics: None,
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
    
    pub async fn load_statistics(&mut self, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        // 从数据库加载统计数据
        let search_result = tagbox_core::search_files_advanced("", None, config).await?;
        let files = &search_result.entries;
        
        // 计算统计数据
        let mut statistics = FileStatistics {
            total_files: files.len(),
            total_size: 0,
            total_categories: 0,
            total_authors: 0,
            recent_imports: 0,
            category_distribution: HashMap::new(),
            filetype_distribution: HashMap::new(),
            author_distribution: HashMap::new(),
            import_timeline: Vec::new(),
        };
        
        let mut categories = std::collections::HashSet::new();
        let mut authors = std::collections::HashSet::new();
        let now = chrono::Utc::now();
        let thirty_days_ago = now - chrono::Duration::days(30);
        
        // 分析每个文件
        for file in files {
            // 总大小（如果有的话）
            // statistics.total_size += file.size.unwrap_or(0);
            
            // 分类统计
            categories.insert(file.category1.clone());
            if let Some(cat2) = &file.category2 {
                categories.insert(cat2.clone());
            }
            if let Some(cat3) = &file.category3 {
                categories.insert(cat3.clone());
            }
            
            // 更新分类分布
            let category_key = format!("{}", file.category1);
            let category_stats = statistics.category_distribution
                .entry(category_key)
                .or_insert(CategoryStats {
                    file_count: 0,
                    total_size: 0,
                    subcategories: Vec::new(),
                    recent_files: 0,
                });
            category_stats.file_count += 1;
            
            // 作者统计
            for author in &file.authors {
                authors.insert(author.clone());
                
                let author_stats = statistics.author_distribution
                    .entry(author.clone())
                    .or_insert(AuthorStats {
                        file_count: 0,
                        total_size: 0,
                        categories: Vec::new(),
                        years: Vec::new(),
                    });
                author_stats.file_count += 1;
                if !author_stats.categories.contains(&file.category1) {
                    author_stats.categories.push(file.category1.clone());
                }
                if let Some(year) = file.year {
                    if !author_stats.years.contains(&year) {
                        author_stats.years.push(year);
                    }
                }
            }
            
            // 文件类型统计
            let extension = file.path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("Unknown")
                .to_lowercase();
            
            let filetype_stats = statistics.filetype_distribution
                .entry(extension)
                .or_insert(FileTypeStats {
                    file_count: 0,
                    total_size: 0,
                    average_size: 0,
                    largest_file: None,
                });
            filetype_stats.file_count += 1;
            
            // 最近导入（基于文件修改时间近似）
            if let Ok(metadata) = std::fs::metadata(&file.path) {
                if let Ok(modified) = metadata.modified() {
                    let modified_datetime = chrono::DateTime::<chrono::Utc>::from(modified);
                    if modified_datetime > thirty_days_ago {
                        statistics.recent_imports += 1;
                    }
                }
            }
        }
        
        statistics.total_categories = categories.len();
        statistics.total_authors = authors.len();
        
        // 计算平均文件大小
        for (_, filetype_stats) in &mut statistics.filetype_distribution {
            if filetype_stats.file_count > 0 {
                filetype_stats.average_size = filetype_stats.total_size / filetype_stats.file_count as u64;
            }
        }
        
        self.statistics = Some(statistics);
        self.update_displays();
        
        Ok(())
    }
    
    fn update_displays(&mut self) {
        if let Some(stats) = self.statistics.clone() {
            self.update_overview(&stats);
            self.update_category_stats(&stats);
            self.update_filetype_stats(&stats);
            self.update_author_stats(&stats);
            self.update_timeline_stats(&stats);
        }
    }
    
    fn update_overview(&mut self, stats: &FileStatistics) {
        self.total_files_output.set_value(&stats.total_files.to_string());
        self.total_size_output.set_value(&Self::format_size(stats.total_size));
        self.categories_count_output.set_value(&stats.total_categories.to_string());
        self.authors_count_output.set_value(&stats.total_authors.to_string());
        
        let recent_text = format!(
            "Recent imports: {} files\nImport rate: {:.1} files/day\nActive period: Last 30 days",
            stats.recent_imports,
            stats.recent_imports as f64 / 30.0
        );
        self.recent_imports_output.set_value(&recent_text);
    }
    
    fn update_category_stats(&mut self, stats: &FileStatistics) {
        self.category_tree.clear();
        
        let mut category_text = String::new();
        category_text.push_str("Category Distribution Analysis\n");
        category_text.push_str("==============================\n\n");
        
        let mut sorted_categories: Vec<_> = stats.category_distribution.iter().collect();
        sorted_categories.sort_by(|a, b| b.1.file_count.cmp(&a.1.file_count));
        
        for (category, cat_stats) in &sorted_categories {
            // 添加到树中
            let tree_label = format!("{} ({})", category, cat_stats.file_count);
            self.category_tree.add(&tree_label);
            
            // 添加到文本显示
            category_text.push_str(&format!(
                "📁 {}\n   Files: {}\n   Size: {}\n   Recent: {}\n\n",
                category,
                cat_stats.file_count,
                Self::format_size(cat_stats.total_size),
                cat_stats.recent_files
            ));
        }
        
        self.category_buffer.set_text(&category_text);
    }
    
    fn update_filetype_stats(&mut self, stats: &FileStatistics) {
        let mut filetype_text = String::new();
        filetype_text.push_str("File Type Distribution Analysis\n");
        filetype_text.push_str("================================\n\n");
        
        let mut sorted_filetypes: Vec<_> = stats.filetype_distribution.iter().collect();
        sorted_filetypes.sort_by(|a, b| b.1.file_count.cmp(&a.1.file_count));
        
        for (extension, type_stats) in &sorted_filetypes {
            let percentage = (type_stats.file_count as f64 / stats.total_files as f64) * 100.0;
            
            filetype_text.push_str(&format!(
                "📄 .{}\n   Count: {} ({:.1}%)\n   Total Size: {}\n   Avg Size: {}\n",
                extension.to_uppercase(),
                type_stats.file_count,
                percentage,
                Self::format_size(type_stats.total_size),
                Self::format_size(type_stats.average_size)
            ));
            
            if let Some(largest) = &type_stats.largest_file {
                filetype_text.push_str(&format!("   Largest: {}\n", largest));
            }
            filetype_text.push('\n');
        }
        
        self.filetype_buffer.set_text(&filetype_text);
    }
    
    fn update_author_stats(&mut self, stats: &FileStatistics) {
        let mut author_text = String::new();
        author_text.push_str("Author Distribution Analysis\n");
        author_text.push_str("=============================\n\n");
        
        let mut sorted_authors: Vec<_> = stats.author_distribution.iter().collect();
        sorted_authors.sort_by(|a, b| b.1.file_count.cmp(&a.1.file_count));
        
        for (author, auth_stats) in sorted_authors.iter().take(50) { // Top 50 authors
            let percentage = (auth_stats.file_count as f64 / stats.total_files as f64) * 100.0;
            
            author_text.push_str(&format!(
                "👤 {}\n   Files: {} ({:.1}%)\n   Categories: {}\n   Years: {}\n   Size: {}\n\n",
                author,
                auth_stats.file_count,
                percentage,
                auth_stats.categories.len(),
                auth_stats.years.len(),
                Self::format_size(auth_stats.total_size)
            ));
        }
        
        if sorted_authors.len() > 50 {
            author_text.push_str(&format!("\n... and {} more authors", sorted_authors.len() - 50));
        }
        
        self.author_buffer.set_text(&author_text);
    }
    
    fn update_timeline_stats(&mut self, stats: &FileStatistics) {
        let mut timeline_text = String::new();
        timeline_text.push_str("Import Timeline Analysis\n");
        timeline_text.push_str("========================\n\n");
        
        timeline_text.push_str("📊 Import Activity Overview:\n");
        timeline_text.push_str(&format!("Total Files: {}\n", stats.total_files));
        timeline_text.push_str(&format!("Recent Activity (30 days): {} files\n", stats.recent_imports));
        timeline_text.push_str(&format!("Average Import Rate: {:.1} files/day\n\n", stats.recent_imports as f64 / 30.0));
        
        timeline_text.push_str("📈 Growth Trends:\n");
        timeline_text.push_str("- File collection growth is tracked by import dates\n");
        timeline_text.push_str("- Recent activity indicates current usage patterns\n");
        timeline_text.push_str("- Consider implementing detailed import logging for better analytics\n\n");
        
        timeline_text.push_str("💡 Recommendations:\n");
        if stats.recent_imports == 0 {
            timeline_text.push_str("- No recent imports detected\n");
            timeline_text.push_str("- Consider importing new content\n");
        } else if stats.recent_imports > 50 {
            timeline_text.push_str("- High import activity detected\n");
            timeline_text.push_str("- Consider organizing new content into categories\n");
        } else {
            timeline_text.push_str("- Moderate import activity\n");
            timeline_text.push_str("- Collection is actively maintained\n");
        }
        
        self.timeline_buffer.set_text(&timeline_text);
    }
    
    fn setup_callbacks(&mut self) {
        // 刷新按钮回调
        let _sender = self.event_sender.clone();
        self.refresh_btn.set_callback(move |_| {
            println!("Refreshing statistics...");
            // TODO: 触发统计数据重新加载
        });
        
        // 导出按钮回调
        self.export_btn.set_callback(|_| {
            Self::export_statistics_report();
        });
        
        // 关闭按钮回调
        self.close_btn.set_callback(move |btn| {
            if let Some(mut window) = btn.window() {
                window.hide();
            }
        });
        
        // Escape键处理
        self.window.set_callback(move |win| {
            if fltk::app::event() == fltk::enums::Event::KeyDown 
                && fltk::app::event_key() == fltk::enums::Key::Escape {
                win.hide();
            }
        });
    }
    
    fn export_statistics_report() {
        // 导出统计报告到文件
        let mut dialog = fltk::dialog::NativeFileChooser::new(
            fltk::dialog::NativeFileChooserType::BrowseSaveFile
        );
        dialog.set_title("Export Statistics Report");
        dialog.set_filter("Text Files\t*.txt\nCSV Files\t*.csv\nHTML Files\t*.html");
        dialog.show();
        
        let filename = dialog.filename();
        if !filename.to_string_lossy().is_empty() {
            // TODO: 实际的导出功能
            println!("Exporting statistics to: {}", filename.display());
            fltk::dialog::message_default(&format!("Statistics report will be exported to:\n{}", filename.display()));
        }
    }
    
    fn format_size(bytes: u64) -> String {
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
            format!("{:.2} {}", size, UNITS[unit_index])
        }
    }
}