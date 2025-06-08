use freya::prelude::*;
use futures::channel::mpsc::UnboundedReceiver;
use crate::router::{Router, Route, use_route};
use crate::components::{TopBar, CategoryTree, FilePreview, ToastContainer, Breadcrumb};
use crate::state::{AppState, FileEntry};

pub fn App() -> Element {
    // 初始化应用状态
    let mut app_state = use_signal(|| None::<AppState>);
    let mut init_error = use_signal(|| None::<String>);
    
    // 异步初始化
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        match AppState::new(None).await {
            Ok(state) => {
                app_state.set(Some(state));
            }
            Err(e) => {
                init_error.set(Some(format!("初始化失败: {}", e)));
                tracing::error!("Failed to initialize app state: {}", e);
            }
        }
    });
    
    // 如果有错误，显示错误界面
    if let Some(error) = init_error.read().as_ref() {
        return rsx! {
            rect {
                width: "100%",
                height: "100%",
                content: "center",
                background: "rgb(250, 250, 250)",
                
                rect {
                    direction: "column",
                    spacing: "20",
                    content: "center",
                    
                    label {
                        font_size: "24",
                        color: "rgb(200, 0, 0)",
                        "初始化失败"
                    }
                    
                    label {
                        font_size: "16",
                        color: "rgb(100, 100, 100)",
                        "{error}"
                    }
                    
                    Button {
                        onpress: move |_| {
                            // 重试
                            init_error.set(None);
                            app_state.set(None);
                        },
                        
                        label { "重试" }
                    }
                }
            }
        };
    }
    
    // 如果还在加载，显示加载界面
    if app_state.read().is_none() {
        return rsx! {
            rect {
                width: "100%",
                height: "100%",
                content: "center",
                background: "rgb(250, 250, 250)",
                
                rect {
                    direction: "column",
                    spacing: "20",
                    content: "center",
                    
                    label {
                        font_size: "20",
                        color: "rgb(100, 100, 100)",
                        "正在加载..."
                    }
                }
            }
        };
    }
    
    // 提供状态上下文
    use_context_provider(|| app_state);
    
    rsx! {
        rect {
            width: "100%",
            height: "100%",
            background: "rgb(250, 250, 250)",
            
            Router {}
            
            // Toast 通知容器
            if let Some(state) = app_state.read().as_ref() {
                ToastContainer {
                    messages: use_signal(|| state.toast_messages.clone()),
                }
            }
        }
    }
}

pub fn MainView() -> Element {
    let app_state = use_context::<Signal<Option<AppState>>>();
    
    rsx! {
        rect {
            width: "100%",
            height: "100%",
            direction: "column",
            
            // 顶部栏
            TopBar {}
            
            // 面包屑导航
            Breadcrumb {}
            
            // 错误消息显示
            if let Some(error) = app_state.read().as_ref().and_then(|s| s.error_message.as_ref()) {
                rect {
                    width: "100%",
                    padding: "10 20",
                    background: "rgb(255, 240, 240)",
                    
                    label {
                        color: "rgb(200, 50, 50)",
                        "{error}"
                    }
                }
            }
            
            // 主内容区域
            rect {
                width: "100%",
                height: "flex",
                direction: "horizontal",
                
                // 左侧面板 - 分类树
                rect {
                    width: "250",
                    height: "100%",
                    background: "rgb(245, 245, 245)",
                    padding: "20",
                    
                    CategoryTree {}
                }
                
                // 中间区域 - 文件列表
                rect {
                    width: "flex",
                    height: "100%",
                    padding: "20",
                    
                    FileList {}
                }
                
                // 右侧面板 - 文件预览
                rect {
                    width: "400",
                    height: "100%",
                    background: "rgb(248, 248, 248)",
                    padding: "20",
                    
                    FilePreview {}
                }
            }
        }
    }
}

fn FileList() -> Element {
    let app_state = use_context::<Signal<Option<AppState>>>();
    
    let files = match app_state.read().as_ref() {
        Some(state) => state.search_results.entries.iter()
            .map(|e| e.clone().into())
            .collect::<Vec<FileEntry>>(),
        None => vec![]
    };
    
    if files.is_empty() {
        return rsx! {
            rect {
                width: "100%",
                height: "100%",
                content: "center",
                
                label {
                    font_size: "18",
                    color: "rgb(150, 150, 150)",
                    "No files found. Click 'Import File' to add files."
                }
            }
        };
    }
    
    rsx! {
        ScrollView {
            width: "100%",
            height: "100%",
            
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
}

#[component]
fn FileCard(file: FileEntry) -> Element {
    let mut app_state = use_context::<Signal<Option<AppState>>>();
    let mut route = use_route();
    
    let is_selected = app_state.read().as_ref()
        .and_then(|s| s.selected_file.as_ref())
        .map(|f| f.id == file.id)
        .unwrap_or(false);
    
    let file_clone = file.clone();
    
    rsx! {
        rect {
            width: "100%",
            padding: "15",
            background: if is_selected { "rgb(240, 240, 255)" } else { "white" },
            corner_radius: "8",
            onclick: move |_| {
                if let Some(state) = app_state.write().as_mut() {
                    state.selected_file = Some(file_clone.clone());
                }
            },
            
            rect {
                spacing: "8",
                direction: "column",
                
                // 标题
                label {
                    font_size: "16",
                    font_weight: "bold",
                    color: "rgb(30, 30, 30)",
                    "{file.title}"
                }
                
                // 标签
                if !file.tags.is_empty() {
                    rect {
                        direction: "horizontal",
                        spacing: "5",
                        
                        for tag in &file.tags {
                            rect {
                                padding: "3 8",
                                background: "rgb(100, 100, 255)",
                                corner_radius: "10",
                                
                                label {
                                    font_size: "11",
                                    color: "white",
                                    "{tag}"
                                }
                            }
                        }
                    }
                }
                
                // 元信息
                rect {
                    direction: "horizontal",
                    spacing: "15",
                    content: "center space",
                    
                    rect {
                        direction: "horizontal",
                        spacing: "15",
                        
                        label {
                            font_size: "12",
                            color: "rgb(120, 120, 120)",
                            "{file.authors.join(\", \")}"
                        }
                        
                        label {
                            font_size: "12", 
                            color: "rgb(150, 150, 150)",
                            "{file.imported_at}"
                        }
                    }
                    
                    // 编辑按钮
                    Button {
                        onpress: move |_| {
                            // 不需要 stop_propagation，Button 会处理
                            let file_id = file.id.clone();
                            route.set(Route::Edit(file_id));
                        },
                        
                        label { "编辑" }
                    }
                }
            }
        }
    }
}