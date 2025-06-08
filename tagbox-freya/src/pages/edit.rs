use freya::prelude::*;
use crate::state::{AppState, FileEntry};
use crate::components::ConfirmDialog;
use crate::router::{Route, use_route};
use futures::{StreamExt, channel::mpsc::UnboundedReceiver};
use tagbox_core::types::ImportMetadata;
use std::collections::HashMap;

#[component]
pub fn EditPage(file_id: String) -> Element {
    let mut app_state = use_context::<Signal<Option<AppState>>>();
    let mut route = use_route();
    
    // 查找要编辑的文件
    let file: Option<FileEntry> = app_state.read().as_ref()
        .and_then(|state| {
            state.search_results.entries.iter()
                .find(|f| f.id.to_string() == file_id)
                .cloned()
                .map(|f| f.into())
        });
    
    match file {
        Some(file) => {
            // 保存原始数据用于重置
            let original_file = file.clone();
            
            let mut title = use_signal(|| file.title.clone());
            let mut authors = use_signal(|| file.authors.join(", "));
            let mut tags = use_signal(|| file.tags.join(", "));
            let mut summary = use_signal(|| file.summary.clone().unwrap_or_default());
            let mut category1 = use_signal(|| file.category.as_ref().map(|c| c.level1.clone()).unwrap_or_default());
            let mut category2 = use_signal(|| file.category.as_ref().and_then(|c| c.level2.clone()).unwrap_or_default());
            let mut category3 = use_signal(|| file.category.as_ref().and_then(|c| c.level3.clone()).unwrap_or_default());
            let mut is_saving = use_signal(|| false);
            let mut save_error = use_signal(|| None::<String>);
            let mut show_delete_dialog = use_signal(|| false);
            let mut is_deleting = use_signal(|| false);
            let mut delete_error = use_signal(|| None::<String>);
            let mut is_extracting = use_signal(|| false);
            let mut extract_error = use_signal(|| None::<String>);
            
            // 保存文件的协程
            let file_id = file.id.clone();
            let save_file_coroutine = use_coroutine(move |mut rx: UnboundedReceiver<()>| {
                let file_id = file_id.clone();
                async move {
                    while let Some(_) = rx.next().await {
                        is_saving.set(true);
                        save_error.set(None);
                        
                        // 构建 ImportMetadata
                        let metadata = ImportMetadata {
                            title: title.read().clone(),
                            authors: authors.read()
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect(),
                            year: None,  // FileEntry 不包含年份字段
                            publisher: None,  // FileEntry 不包含出版商字段
                            source: None,  // FileEntry 不包含来源字段
                            category1: category1.read().clone(),
                            category2: if category2.read().is_empty() { None } else { Some(category2.read().clone()) },
                            category3: if category3.read().is_empty() { None } else { Some(category3.read().clone()) },
                            tags: tags.read()
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect(),
                            summary: if summary.read().is_empty() { None } else { Some(summary.read().clone()) },
                            full_text: None,  // 编辑页面不修改全文
                            additional_info: HashMap::new(),
                            file_metadata: None,
                            type_metadata: None,
                        };
                        
                        let result = if let Some(state) = app_state.read().as_ref() {
                            state.service.update_file(&file_id, metadata).await
                        } else {
                            Err(anyhow::anyhow!("App state not available"))
                        };
                        
                        match result {
                            Ok(_) => {
                                tracing::info!("File updated successfully");
                                if let Some(state) = app_state.write().as_mut() {
                                    state.show_success("文件保存成功");
                                }
                            },
                            Err(e) => {
                                save_error.set(Some(format!("保存失败: {}", e)));
                                tracing::error!("Failed to save file: {}", e);
                            }
                        }
                        
                        is_saving.set(false);
                    }
                }
            });
            
            // 删除文件的协程
            let file_id_for_delete = file.id.clone();
            let delete_file_coroutine = use_coroutine(move |mut rx: UnboundedReceiver<()>| {
                let file_id = file_id_for_delete.clone();
                async move {
                    while let Some(_) = rx.next().await {
                        is_deleting.set(true);
                        delete_error.set(None);
                        
                        let result = if let Some(state) = app_state.read().as_ref() {
                            state.service.delete_file(&file_id).await
                        } else {
                            Err(anyhow::anyhow!("App state not available"))
                        };
                        
                        match result {
                            Ok(_) => {
                                tracing::info!("File deleted successfully");
                                if let Some(state) = app_state.write().as_mut() {
                                    state.show_success("文件删除成功");
                                }
                                route.set(Route::Main);
                            },
                            Err(e) => {
                                delete_error.set(Some(format!("删除失败: {}", e)));
                                tracing::error!("Failed to delete file: {}", e);
                            }
                        }
                        
                        is_deleting.set(false);
                    }
                }
            });
            
            // 重新提取元数据的协程
            let file_path = file.path.clone();
            let extract_metadata_coroutine = use_coroutine(move |mut rx: UnboundedReceiver<()>| {
                let file_path = file_path.clone();
                async move {
                    while let Some(_) = rx.next().await {
                        is_extracting.set(true);
                        extract_error.set(None);
                        
                        let path = std::path::PathBuf::from(&file_path);
                        let result = if let Some(state) = app_state.read().as_ref() {
                            state.service.extract_metadata(&path).await
                        } else {
                            Err(anyhow::anyhow!("App state not available"))
                        };
                        
                        match result {
                            Ok(metadata) => {
                                // 更新表单字段
                                title.set(metadata.title.clone());
                                authors.set(metadata.authors.join(", "));
                                tags.set(metadata.tags.join(", "));
                                summary.set(metadata.summary.clone().unwrap_or_default());
                                category1.set(metadata.category1.clone());
                                category2.set(metadata.category2.clone().unwrap_or_default());
                                category3.set(metadata.category3.clone().unwrap_or_default());
                                
                                tracing::info!("Metadata re-extracted successfully");
                                if let Some(state) = app_state.write().as_mut() {
                                    state.show_success("元数据提取成功");
                                }
                            },
                            Err(e) => {
                                extract_error.set(Some(format!("元数据提取失败: {}", e)));
                                tracing::error!("Failed to extract metadata: {}", e);
                            }
                        }
                        
                        is_extracting.set(false);
                    }
                }
            });
            
            rsx! {
                ScrollView {
                    rect {
                        width: "100%",
                        padding: "40",
                        direction: "column",
                        spacing: "30",
                        
                        // 页面标题
                        rect {
                            direction: "horizontal",
                            content: "center start",
                            
                            label {
                                font_size: "28",
                                font_weight: "bold",
                                width: "flex",
                                "Edit File"
                            }
                            
                            // 返回按钮
                            Button {
                                onpress: move |_| {
                                    route.set(Route::Main);
                                },
                                
                                label { "← Back" }
                            }
                        }
                        
                        // 错误消息显示
                        if let Some(error) = save_error.read().as_ref() {
                            rect {
                                width: "100%",
                                padding: "15",
                                background: "rgb(255, 240, 240)",
                                corner_radius: "4",
                                
                                label {
                                    color: "rgb(200, 50, 50)",
                                    "{error}"
                                }
                            }
                        }
                        
                        // 提取错误消息显示
                        if let Some(error) = extract_error.read().as_ref() {
                            rect {
                                width: "100%",
                                padding: "15",
                                background: "rgb(255, 243, 224)",
                                corner_radius: "4",
                                
                                label {
                                    color: "rgb(180, 83, 9)",
                                    "{error}"
                                }
                            }
                        }
                        
                        // 文件信息
                        rect {
                            width: "100%",
                            padding: "20",
                            background: "rgb(250, 250, 250)",
                            corner_radius: "8",
                            direction: "column",
                            spacing: "10",
                            
                            label {
                                font_size: "12",
                                color: "rgb(120, 120, 120)",
                                "File Path"
                            }
                            
                            label {
                                font_size: "14",
                                color: "rgb(80, 80, 80)",
                                "{file.path}"
                            }
                            
                            rect {
                                direction: "horizontal",
                                spacing: "40",
                                margin: "10 0 0 0",
                                
                                label {
                                    font_size: "12",
                                    color: "rgb(120, 120, 120)",
                                    "ID: {file.id}"
                                }
                                
                                label {
                                    font_size: "12",  
                                    color: "rgb(120, 120, 120)",
                                    "Size: {format_file_size(file.size)}"
                                }
                                
                                label {
                                    font_size: "12",
                                    color: "rgb(120, 120, 120)",
                                    "Modified: {file.modified_at}"
                                }
                            }
                        }
                        
                        // 元数据表单
                        rect {
                            width: "100%",
                            direction: "column",
                            spacing: "20",
                            
                            // 标题
                            EditField {
                                label: "Title",
                                value: title,
                            }
                            
                            // 作者
                            EditField {
                                label: "Authors",
                                value: authors,
                            }
                            
                            // 标签
                            EditField {
                                label: "Tags",
                                value: tags,
                            }
                            
                            // 摘要
                            rect {
                                direction: "column",
                                spacing: "8",
                                
                                label {
                                    font_size: "14",
                                    font_weight: "bold",
                                    "Summary"
                                }
                                
                                rect {
                                    width: "100%",
                                    height: "200",
                                    background: "rgb(245, 245, 245)",
                                    corner_radius: "4",
                                    padding: "10",
                                    
                                    Input {
                                        width: "100%",
                                        value: "{summary}",
                                        onchange: move |e: String| {
                                            summary.set(e);
                                        },
                                    }
                                }
                            }
                            
                            // 分类
                            rect {
                                direction: "column",
                                spacing: "8",
                                
                                label {
                                    font_size: "14",
                                    font_weight: "bold",
                                    "Category"
                                }
                                
                                rect {
                                    direction: "horizontal",
                                    spacing: "10",
                                    
                                    CategorySelector {
                                        level: 1,
                                        value: category1,
                                    }
                                    
                                    CategorySelector {
                                        level: 2,
                                        value: category2,
                                    }
                                    
                                    CategorySelector {
                                        level: 3,
                                        value: category3,
                                    }
                                }
                            }
                        }
                        
                        // 关联文件
                        LinkedFilesSection {
                            file_id: file.id.clone()
                        }
                        
                        // 操作按钮
                        rect {
                            direction: "horizontal",
                            spacing: "10",
                            margin: "20 0 0 0",
                            
                            Button {
                                onpress: move |_| {
                                    extract_metadata_coroutine.send(());
                                },
                                
                                label { if is_extracting() { "Extracting..." } else { "Re-extract Metadata" } }
                            }
                            
                            Button {
                                onpress: move |_| {
                                    // 重置所有字段到原始值
                                    title.set(original_file.title.clone());
                                    authors.set(original_file.authors.join(", "));
                                    tags.set(original_file.tags.join(", "));
                                    summary.set(original_file.summary.clone().unwrap_or_default());
                                    category1.set(original_file.category.as_ref().map(|c| c.level1.clone()).unwrap_or_default());
                                    category2.set(original_file.category.as_ref().and_then(|c| c.level2.clone()).unwrap_or_default());
                                    category3.set(original_file.category.as_ref().and_then(|c| c.level3.clone()).unwrap_or_default());
                                    
                                    // 清除错误消息
                                    save_error.set(None);
                                    delete_error.set(None);
                                    
                                    tracing::info!("Reset to original values");
                                },
                                
                                label { "Reset to Original" }
                            }
                            
                            rect { width: "flex" }
                            
                            Button {
                                onpress: move |_| {
                                    show_delete_dialog.set(true);
                                },
                                
                                label { if is_deleting() { "Deleting..." } else { "Delete" } }
                            }
                            
                            Button {
                                onpress: move |_| {
                                    route.set(Route::Main);
                                },
                                
                                label { "Cancel" }
                            }
                            
                            Button {
                                onpress: move |_| {
                                    if !is_saving() {
                                        save_file_coroutine.send(());
                                    }
                                },
                                
                                label { if is_saving() { "Saving..." } else { "Save" } }
                            }
                        }
                    }
                }
                
                // 删除确认对话框
                ConfirmDialog {
                    title: "删除文件",
                    message: "确定要删除这个文件吗？此操作无法撤销。",
                    is_open: show_delete_dialog,
                    on_confirm: move |_| {
                        delete_file_coroutine.send(());
                    },
                    on_cancel: move |_| {
                        // 取消时不做任何操作
                    },
                }
                
                // 删除错误提示
                if let Some(error) = delete_error.read().as_ref() {
                    rect {
                        position: "absolute",
                        position_bottom: "20",
                        position_left: "50%",
                        offset_x: "-200",
                        width: "400",
                        padding: "15",
                        background: "rgb(254, 226, 226)",
                        corner_radius: "4",
                        
                        label {
                            color: "rgb(185, 28, 28)",
                            "{error}"
                        }
                    }
                }
            }
        },
        None => rsx! {
            rect {
                width: "100%",
                height: "100%",
                content: "center",
                
                label {
                    font_size: "18",
                    color: "rgb(150, 150, 150)",
                    "File not found"
                }
            }
        }
    }
}

#[component]
fn EditField(
    label: &'static str,
    mut value: Signal<String>,
) -> Element {
    rsx! {
        rect {
            direction: "column",
            spacing: "8",
            
            label {
                font_size: "14",
                font_weight: "bold",
                "{label}"
            }
            
            Input {
                width: "100%",
                value: "{value}",
                onchange: move |e: String| {
                    value.set(e);
                },
            }
        }
    }
}

#[component]
fn CategorySelector(
    level: u8,
    mut value: Signal<String>,
) -> Element {
    let display_text = if value.read().is_empty() {
        format!("Select Level {}", level)
    } else {
        value.read().clone()
    };
    
    rsx! {
        rect {
            width: "180",
            height: "40",
            padding: "10",
            background: "rgb(245, 245, 245)",
            corner_radius: "4",
            onclick: move |_| {
                // TODO: 显示分类选择器
                tracing::info!("Show category selector for level {}", level);
            },
            
            label {
                color: if value.read().is_empty() { "rgb(150, 150, 150)" } else { "rgb(50, 50, 50)" },
                "{display_text}"
            }
        }
    }
}

#[component]
fn LinkedFilesSection(file_id: String) -> Element {
    // TODO: 从数据库获取关联文件
    let linked_files: Vec<crate::state::FileEntry> = vec![];
    
    rsx! {
        rect {
            width: "100%",
            direction: "column",
            spacing: "15",
            
            label {
                font_size: "18",
                font_weight: "bold",
                "Linked Files"
            }
            
            if linked_files.is_empty() {
                rect {
                    width: "100%",
                    padding: "20",
                    background: "rgb(250, 250, 250)",
                    corner_radius: "8",
                    content: "center",
                    
                    label {
                        color: "rgb(150, 150, 150)",
                        "No linked files"
                    }
                }
            } else {
                rect {
                    width: "100%",
                    direction: "column",
                    spacing: "10",
                    
                    // TODO: 显示关联文件列表
                }
            }
            
            Button {
                onclick: move |_| {
                    // TODO: 添加关联文件
                    tracing::info!("Add linked file");
                },
                
                label { "Add Link" }
            }
        }
    }
}

fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    format!("{:.2} {}", size, UNITS[unit_index])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0.00 B");
        assert_eq!(format_file_size(512), "512.00 B");
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1536), "1.50 KB");
        assert_eq!(format_file_size(1048576), "1.00 MB");
        assert_eq!(format_file_size(1073741824), "1.00 GB");
        assert_eq!(format_file_size(1099511627776), "1.00 TB");
    }

    #[test]
    fn test_metadata_parsing() {
        // 测试作者解析
        let authors_str = "Author 1, Author 2, , Author 3";
        let authors: Vec<String> = authors_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        assert_eq!(authors, vec!["Author 1", "Author 2", "Author 3"]);

        // 测试标签解析
        let tags_str = "tag1,tag2,,tag3,   tag4   ";
        let tags: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        assert_eq!(tags, vec!["tag1", "tag2", "tag3", "tag4"]);
    }

    #[test]
    fn test_import_metadata_construction() {
        let title = "Test Title";
        let authors = vec!["Author 1".to_string(), "Author 2".to_string()];
        let category1 = "Category 1";
        let category2 = "Category 2";
        let category3 = "";
        let tags = vec!["tag1".to_string(), "tag2".to_string()];
        let summary = "Test summary";

        let metadata = ImportMetadata {
            title: title.to_string(),
            authors: authors.clone(),
            year: None,
            publisher: None,
            source: None,
            category1: category1.to_string(),
            category2: if category2.is_empty() { None } else { Some(category2.to_string()) },
            category3: if category3.is_empty() { None } else { Some(category3.to_string()) },
            tags: tags.clone(),
            summary: if summary.is_empty() { None } else { Some(summary.to_string()) },
            full_text: None,
            additional_info: HashMap::new(),
            file_metadata: None,
            type_metadata: None,
        };

        assert_eq!(metadata.title, "Test Title");
        assert_eq!(metadata.authors, authors);
        assert_eq!(metadata.category1, "Category 1");
        assert_eq!(metadata.category2, Some("Category 2".to_string()));
        assert_eq!(metadata.category3, None);
        assert_eq!(metadata.tags, tags);
        assert_eq!(metadata.summary, Some("Test summary".to_string()));
    }
}