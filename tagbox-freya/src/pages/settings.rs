use freya::prelude::*;
use crate::state::AppState;
use crate::router::{Route, use_route};
use std::path::PathBuf;
use futures::channel::mpsc::UnboundedReceiver;
use futures::StreamExt;

pub fn SettingsPage() -> Element {
    let mut app_state = use_context::<Signal<Option<AppState>>>();
    let mut route = use_route();
    
    // 配置相关状态
    let mut config_path = use_signal(|| String::new());
    let mut config_content = use_signal(|| String::new());
    let mut is_loading = use_signal(|| false);
    let mut save_message = use_signal(|| None::<String>);
    let mut error_message = use_signal(|| None::<String>);
    
    // 加载当前配置
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        is_loading.set(true);
        
        if let Some(state) = app_state.read().as_ref() {
            // 获取配置路径
            let path = state.service.config_path().unwrap_or_default();
            config_path.set(path.to_string_lossy().to_string());
            
            // 读取配置内容
            match tokio::fs::read_to_string(&path).await {
                Ok(content) => {
                    config_content.set(content);
                }
                Err(e) => {
                    error_message.set(Some(format!("读取配置文件失败: {}", e)));
                }
            }
        }
        
        is_loading.set(false);
    });
    
    // 保存配置的协程
    let save_config_coroutine = use_coroutine(move |mut rx: UnboundedReceiver<()>| async move {
        while let Some(_) = rx.next().await {
            is_loading.set(true);
            save_message.set(None);
            error_message.set(None);
            
            let path = PathBuf::from(config_path.read().clone());
            let content = config_content.read().clone();
            
            match tokio::fs::write(&path, &content).await {
                Ok(_) => {
                    save_message.set(Some("配置已保存".to_string()));
                    
                    // 通知服务重新加载配置
                    // TODO: 实现配置重新加载
                    save_message.set(Some("配置已保存（需要重启应用以生效）".to_string()));
                }
                Err(e) => {
                    error_message.set(Some(format!("保存配置失败: {}", e)));
                }
            }
            
            is_loading.set(false);
        }
    });
    
    // 初始化新配置文件的协程
    let init_config_coroutine = use_coroutine(move |mut rx: UnboundedReceiver<String>| async move {
        while let Some(path) = rx.next().await {
            is_loading.set(true);
            
            // 创建默认配置内容
            let default_config = r#"# TagBox Configuration
# TagBox 配置文件

[database]
# Database file path
# 数据库文件路径
path = "tagbox_data/tagbox.db"

[import]
# Default paths for import
# 默认导入路径
default_paths = ["~/Documents", "~/Downloads"]

# File size limit in MB
# 文件大小限制（MB）
max_file_size = 100

[search]
# Maximum search results
# 最大搜索结果数
max_results = 100

# Enable fuzzy search
# 启用模糊搜索
fuzzy_search = true

[ui]
# Theme (light/dark)
# 主题（light/dark）
theme = "light"

# Language (en/zh)
# 语言（en/zh）
language = "zh"
"#;
            
            match tokio::fs::write(&path, default_config).await {
                Ok(_) => {
                    config_path.set(path.clone());
                    config_content.set(default_config.to_string());
                    save_message.set(Some("新配置文件已创建".to_string()));
                }
                Err(e) => {
                    error_message.set(Some(format!("创建配置文件失败: {}", e)));
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
                        "Settings"
                    }
                    
                    Button {
                        onpress: move |_| {
                            route.set(Route::Main);
                        },
                        
                        label { "← Back" }
                    }
                }
                
                // 成功消息
                if let Some(message) = save_message.read().as_ref() {
                    rect {
                        width: "100%",
                        padding: "15",
                        background: "rgb(240, 255, 240)",
                        corner_radius: "8",
                        
                        label {
                            color: "rgb(50, 200, 50)",
                            "{message}"
                        }
                    }
                }
                
                // 错误消息
                if let Some(error) = error_message.read().as_ref() {
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
                
                // 配置文件路径
                rect {
                    width: "100%",
                    direction: "column",
                    spacing: "10",
                    
                    label {
                        font_size: "16",
                        font_weight: "bold",
                        "Configuration File Path"
                    }
                    
                    rect {
                        direction: "horizontal",
                        spacing: "10",
                        
                        Input {
                            width: "fill",
                            value: "{config_path}",
                            onchange: move |e: String| {
                                config_path.set(e);
                            },
                        }
                        
                        Button {
                            onpress: move |_| {
                                // 选择文件对话框（需要实现）
                                tracing::info!("Open file dialog");
                            },
                            
                            label { "Browse" }
                        }
                        
                        Button {
                            onpress: move |_| {
                                let new_path = format!("{}/tagbox.toml", std::env::current_dir().unwrap().display());
                                init_config_coroutine.send(new_path);
                            },
                            
                            label { "New Config" }
                        }
                    }
                }
                
                // 配置内容编辑器
                rect {
                    width: "100%",
                    direction: "column",
                    spacing: "10",
                    
                    label {
                        font_size: "16",
                        font_weight: "bold",
                        "Configuration Content"
                    }
                    
                    rect {
                        width: "100%",
                        height: "400",
                        background: "rgb(245, 245, 245)",
                        corner_radius: "4",
                        padding: "10",
                        border: "1 solid rgb(220, 220, 220)",
                        
                        Input {
                            width: "100%",
                            value: "{config_content}",
                            onchange: move |e: String| {
                                config_content.set(e);
                            },
                        }
                    }
                }
                
                // 操作按钮
                rect {
                    width: "100%",
                    direction: "horizontal",
                    spacing: "10",
                    content: "center end",
                    
                    Button {
                        onpress: move |_| {
                            // 重新加载原始内容
                            error_message.set(None);
                            save_message.set(None);
                        },
                        
                        label { "Reset" }
                    }
                    
                    Button {
                        onpress: move |_| {
                            save_config_coroutine.send(());
                        },
                        
                        label { if is_loading() { "Saving..." } else { "Save Config" } }
                    }
                }
                
                // 数据库管理
                rect {
                    width: "100%",
                    direction: "column",
                    spacing: "10",
                    margin: "20 0 0 0",
                    
                    label {
                        font_size: "18",
                        font_weight: "bold",
                        "Database Management"
                    }
                    
                    rect {
                        direction: "horizontal",
                        spacing: "10",
                        
                        Button {
                            onpress: move |_| {
                                tracing::info!("Rebuild search index");
                            },
                            
                            label { "Rebuild Search Index" }
                        }
                        
                        Button {
                            onpress: move |_| {
                                tracing::info!("Export database");
                            },
                            
                            label { "Export Database" }
                        }
                        
                        Button {
                            onpress: move |_| {
                                tracing::info!("Backup database");
                            },
                            
                            label { "Backup Database" }
                        }
                    }
                }
            }
        }
    }
}