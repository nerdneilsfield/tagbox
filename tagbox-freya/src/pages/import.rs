use freya::prelude::*;
use crate::components::{DragDropArea, SelectedFileDisplay};
use crate::state::AppState;
use crate::router::{Route, use_route};
use std::path::PathBuf;
use futures::channel::mpsc::UnboundedReceiver;
use futures::StreamExt;
use tagbox_core::types::ImportMetadata;

pub fn ImportPage() -> Element {
    let mut app_state = use_context::<Signal<Option<AppState>>>();
    let mut selected_file = use_signal(|| None::<PathBuf>);
    let mut download_url = use_signal(|| String::new());
    let mut title = use_signal(|| String::new());
    let mut authors = use_signal(|| String::new());
    let mut year = use_signal(|| String::new());
    let mut publisher = use_signal(|| String::new());
    let mut tags = use_signal(|| String::new());
    let mut summary = use_signal(|| String::new());
    let mut category1 = use_signal(|| String::new());
    let mut category2 = use_signal(|| String::new());
    let mut category3 = use_signal(|| String::new());
    let mut is_loading = use_signal(|| false);
    let mut error_message = use_signal(|| None::<String>);
    
    let file_selected = selected_file.read().is_some();
    
    // 元数据提取协程
    let extract_metadata_coroutine = use_coroutine(move |mut rx: UnboundedReceiver<PathBuf>| async move {
        while let Some(path) = rx.next().await {
            is_loading.set(true);
            error_message.set(None);
            
            if let Some(state) = app_state.read().as_ref() {
                match state.service.extract_metadata(&path).await {
                    Ok(metadata) => {
                        // 填充表单
                        title.set(metadata.title);
                        authors.set(metadata.authors.join(", "));
                        if let Some(y) = metadata.year {
                            year.set(y.to_string());
                        }
                        if let Some(p) = metadata.publisher {
                            publisher.set(p);
                        }
                        tags.set(metadata.tags.join(", "));
                        if let Some(s) = metadata.summary {
                            summary.set(s);
                        }
                        category1.set(metadata.category1);
                        if let Some(c2) = metadata.category2 {
                            category2.set(c2);
                        }
                        if let Some(c3) = metadata.category3 {
                            category3.set(c3);
                        }
                    }
                    Err(e) => {
                        error_message.set(Some(format!("元数据提取失败: {}", e)));
                    }
                }
            }
            
            is_loading.set(false);
        }
    });
    
    // 文件导入协程
    let import_file_coroutine = use_coroutine(move |mut rx: UnboundedReceiver<(PathBuf, bool)>| async move {
        while let Some((path, _move_file)) = rx.next().await {
            is_loading.set(true);
            error_message.set(None);
            
            // 构建元数据
            let metadata = ImportMetadata {
                title: title.read().clone(),
                authors: authors.read().split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
                year: year.read().parse().ok(),
                publisher: if publisher.read().is_empty() { None } else { Some(publisher.read().clone()) },
                source: None,
                category1: if category1.read().is_empty() { "未分类".to_string() } else { category1.read().clone() },
                category2: if category2.read().is_empty() { None } else { Some(category2.read().clone()) },
                category3: if category3.read().is_empty() { None } else { Some(category3.read().clone()) },
                tags: tags.read().split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
                summary: if summary.read().is_empty() { None } else { Some(summary.read().clone()) },
                full_text: None,
                additional_info: std::collections::HashMap::new(),
                file_metadata: None,
                type_metadata: None,
            };
            
            // 执行导入
            let import_result = if let Some(state) = app_state.read().as_ref() {
                state.service.import_file(&path, Some(metadata)).await
            } else {
                Err(anyhow::anyhow!("应用状态未初始化"))
            };
            
            match import_result {
                Ok(_) => {
                    // 导入成功，刷新文件列表
                    if let Some(state) = app_state.write().as_mut() {
                        let _ = state.search("*").await;
                    }
                    
                    // 清空表单
                    selected_file.set(None);
                    title.set(String::new());
                    authors.set(String::new());
                    year.set(String::new());
                    publisher.set(String::new());
                    tags.set(String::new());
                    summary.set(String::new());
                    category1.set(String::new());
                    category2.set(String::new());
                    category3.set(String::new());
                    
                    tracing::info!("文件导入成功");
                }
                Err(e) => {
                    error_message.set(Some(format!("文件导入失败: {}", e)));
                }
            }
            
            is_loading.set(false);
        }
    });
    
    rsx! {
        ScrollView {
            rect {
                width: "100%",
                padding: "40",
                direction: "column",
                spacing: "30",
                
                // 页面标题和返回按钮
                rect {
                    width: "100%",
                    direction: "horizontal",
                    content: "center space",
                    margin: "0 0 20 0",
                    
                    label {
                        font_size: "28",
                        font_weight: "bold",
                        "Import New File"
                    }
                    
                    Button {
                        onpress: move |_| {
                            let mut route = use_route();
                            route.set(Route::Main);
                        },
                        
                        label { "← Back" }
                    }
                }
                
                // 错误消息显示
                if let Some(error) = error_message.read().clone() {
                    rect {
                        width: "100%",
                        padding: "15",
                        background: "rgb(255, 240, 240)",
                        corner_radius: "8",
                        
                        label {
                            color: "rgb(200, 50, 50)",
                            "{error}"
                        }
                    }
                }
                
                // 加载指示器
                if is_loading() {
                    rect {
                        width: "100%",
                        padding: "15",
                        background: "rgb(240, 240, 255)",
                        corner_radius: "8",
                        
                        label {
                            color: "rgb(50, 50, 200)",
                            "处理中..."
                        }
                    }
                }
                
                // 文件选择区域
                rect {
                    width: "100%",
                    direction: "column",
                    spacing: "20",
                    
                    label {
                        font_size: "18",
                        font_weight: "bold",
                        "File Selection"
                    }
                    
                    // 拖放区域
                    DragDropArea {
                        onfile: move |path: PathBuf| {
                            selected_file.set(Some(path));
                        }
                    }
                    
                    // 显示选中的文件
                    SelectedFileDisplay {
                        file_path: selected_file.read().clone(),
                        onremove: move |_| {
                            selected_file.set(None);
                        }
                    }
                    
                    // URL 下载区域
                    rect {
                        width: "100%",
                        direction: "horizontal",
                        spacing: "10",
                        
                        Input {
                            width: "flex",
                            placeholder: "Or enter URL to download",
                            value: "{download_url}",
                            onchange: move |e: String| {
                                download_url.set(e);
                            },
                        }
                        
                        Button {
                            onpress: move |_| {
                                // TODO: 下载文件
                                tracing::info!("Download from URL: {}", download_url.read());
                            },
                            
                            label { "Download" }
                        }
                    }
                }
                
                // 元数据表单
                if file_selected {
                    rect {
                        width: "100%",
                        direction: "column",
                        spacing: "20",
                        
                        label {
                            font_size: "18",
                            font_weight: "bold",
                            "File Metadata"
                        }
                        
                        // 标题
                        MetadataField {
                            label: "Title",
                            placeholder: "Enter file title",
                            value: title,
                        }
                        
                        // 作者
                        MetadataField {
                            label: "Authors",
                            placeholder: "Enter authors (comma separated)",
                            value: authors,
                        }
                        
                        // 年份和出版社
                        rect {
                            direction: "horizontal",
                            spacing: "20",
                            
                            rect {
                                width: "200",
                                direction: "column",
                                spacing: "8",
                                
                                label {
                                    font_size: "14",
                                    font_weight: "bold",
                                    "Year"
                                }
                                
                                Input {
                                    width: "100%",
                                    placeholder: "YYYY",
                                    value: "{year}",
                                    onchange: move |e: String| {
                                        year.set(e);
                                    },
                                }
                            }
                            
                            rect {
                                width: "flex",
                                direction: "column",
                                spacing: "8",
                                
                                label {
                                    font_size: "14",
                                    font_weight: "bold",
                                    "Publisher"
                                }
                                
                                Input {
                                    width: "100%",
                                    placeholder: "Enter publisher",
                                    value: "{publisher}",
                                    onchange: move |e: String| {
                                        publisher.set(e);
                                    },
                                }
                            }
                        }
                        
                        // 标签
                        MetadataField {
                            label: "Tags",
                            placeholder: "Enter tags (comma separated)",
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
                                height: "150",
                                background: "rgb(245, 245, 245)",
                                corner_radius: "4",
                                padding: "10",
                                
                                Input {
                                    width: "100%",
                                    placeholder: "Enter file summary...",
                                    value: "{summary}",
                                    onchange: move |e: String| {
                                        summary.set(e);
                                    },
                                }
                            }
                        }
                        
                        // 分类选择
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
                                
                                CategoryDropdown {
                                    level: 1,
                                    value: category1,
                                }
                                
                                CategoryDropdown {
                                    level: 2,
                                    value: category2,
                                }
                                
                                CategoryDropdown {
                                    level: 3,
                                    value: category3,
                                }
                            }
                        }
                        
                        // 操作按钮
                        rect {
                            direction: "horizontal",
                            spacing: "10",
                            margin: "20 0 0 0",
                            
                            Button {
                                onpress: move |_| {
                                    if let Some(path) = selected_file.read().clone() {
                                        extract_metadata_coroutine.send(path);
                                    }
                                },
                                
                                label { "Extract Metadata" }
                            }
                            
                            Button {
                                onpress: move |_| {
                                    if let Some(path) = selected_file.read().clone() {
                                        import_file_coroutine.send((path, true));
                                    }
                                },
                                
                                label { "Import and Move" }
                            }
                            
                            Button {
                                onpress: move |_| {
                                    if let Some(path) = selected_file.read().clone() {
                                        import_file_coroutine.send((path, false));
                                    }
                                },
                                
                                label { "Import and Keep Original" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn MetadataField(
    label: &'static str,
    placeholder: &'static str,
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
                placeholder: "{placeholder}",
                value: "{value}",
                onchange: move |e: String| {
                    value.set(e);
                },
            }
        }
    }
}

#[component]
fn CategoryDropdown(
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
                // TODO: 显示下拉菜单
                tracing::info!("Show category dropdown for level {}", level);
            },
            
            label {
                color: if value.read().is_empty() { "rgb(150, 150, 150)" } else { "rgb(50, 50, 50)" },
                "{display_text}"
            }
        }
    }
}