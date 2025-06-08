use freya::prelude::*;
use crate::router::Router;
use crate::components::{TopBar, CategoryTree, FilePreview};
use crate::state::{AppState, FileEntry};
use crate::utils::api::TagBoxApi;
use std::sync::Arc;

pub fn App() -> Element {
    // 初始化应用状态
    let mut app_state = use_context_provider(|| Signal::new(AppState::new()));
    
    // 初始化 API
    let mut api_initialized = use_signal(|| false);
    let mut api = use_signal(|| None::<Arc<TagBoxApi>>);
    
    use_effect(move || {
        if !api_initialized() {
            spawn(async move {
                match TagBoxApi::new(None).await {
                    Ok(tagbox_api) => {
                        if let Err(e) = tagbox_api.init_database().await {
                            tracing::error!("Failed to initialize database: {}", e);
                        } else {
                            api.set(Some(Arc::new(tagbox_api)));
                            api_initialized.set(true);
                            
                            // 加载初始数据
                            if let Some(api) = api.read().as_ref() {
                                match api.list_files(Some(100)).await {
                                    Ok(files) => {
                                        let converted_files: Vec<FileEntry> = files.into_iter()
                                            .map(|f| f.into())
                                            .collect();
                                        app_state.write().files = converted_files;
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to load files: {}", e);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to create API: {}", e);
                    }
                }
            });
        }
    });
    
    // 提供 API 上下文
    use_context_provider(|| api);

    rsx! {
        Router {}
    }
}

pub fn MainView() -> Element {
    rsx! {
        rect {
            width: "100%",
            height: "100%",
            background: "rgb(245, 245, 245)",
            direction: "column",
            
            // 顶部栏
            TopBar {}
            
            // 主内容区域
            rect {
                width: "100%",
                height: "calc(100% - 60)",
                direction: "horizontal",
                
                // 左侧分类树
                rect {
                    width: "300",
                    height: "100%",
                    background: "white",
                    shadow: "0 0 10 0 rgb(0, 0, 0, 10)",
                    padding: "10",
                    
                    CategoryTree {}
                }
                
                // 中间文件列表
                rect {
                    width: "fill",
                    height: "100%",
                    padding: "20",
                    
                    ScrollView {
                        FileList {}
                    }
                }
                
                // 右侧预览面板
                rect {
                    width: "400",
                    height: "100%",
                    background: "white",
                    shadow: "0 0 10 0 rgb(0, 0, 0, 10)",
                    padding: "20",
                    
                    FilePreview {}
                }
            }
        }
    }
}

fn FileList() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let files = app_state.read().files.clone();
    
    rsx! {
        rect {
            width: "100%",
            direction: "column",
            spacing: "10",
            
            for file in files {
                FileCard {
                    key: "{file.id}",
                    file: file.clone()
                }
            }
        }
    }
}

#[component]
fn FileCard(file: FileEntry) -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let is_selected = app_state.read().selected_file.as_ref() == Some(&file);
    
    rsx! {
        rect {
            width: "100%",
            padding: "15",
            background: if is_selected { "rgb(240, 240, 255)" } else { "white" },
            corner_radius: "8",
            shadow: "0 2 8 0 rgb(0, 0, 0, 8)",
            onclick: move |_| {
                app_state.write().selected_file = Some(file.clone());
            },
            
            rect {
                direction: "column",
                spacing: "8",
                
                // 标题
                label {
                    font_size: "16",
                    font_weight: "bold",
                    color: "rgb(30, 30, 30)",
                    "{file.title}"
                }
                
                // 标签
                rect {
                    direction: "horizontal",
                    spacing: "5",
                    
                    for tag in &file.tags {
                        rect {
                            padding: "4 8",
                            background: "rgb(100, 100, 255)",
                            corner_radius: "12",
                            
                            label {
                                font_size: "12",
                                color: "white",
                                "{tag}"
                            }
                        }
                    }
                }
                
                // 摘要
                if let Some(summary) = &file.summary {
                    label {
                        font_size: "14",
                        color: "rgb(100, 100, 100)",
                        max_lines: "2",
                        text_overflow: "ellipsis",
                        "{summary}"
                    }
                }
            }
        }
    }
}