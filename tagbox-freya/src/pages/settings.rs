use freya::prelude::*;
use crate::state::AppState;
use crate::router::{Route, use_route};
use crate::components::CustomButton;
use std::path::PathBuf;
use futures::channel::mpsc::UnboundedReceiver;
use futures::StreamExt;

pub fn SettingsPage() -> Element {
    let app_state = use_context::<Signal<Option<AppState>>>();
    let mut route = use_route();
    
    let mut config_path = use_signal(|| String::from("tagbox.toml"));
    let mut config_content = use_signal(|| String::new());
    let mut is_loading = use_signal(|| false);
    let mut save_message = use_signal(|| None::<String>);
    let mut error_message = use_signal(|| None::<String>);
    
    // 加载配置文件的协程
    let load_config_coroutine = use_coroutine(move |mut rx: UnboundedReceiver<String>| async move {
        while let Some(path) = rx.next().await {
            is_loading.set(true);
            error_message.set(None);
            save_message.set(None);
            
            match tokio::fs::read_to_string(&path).await {
                Ok(content) => {
                    config_content.set(content);
                    config_path.set(path);
                }
                Err(e) => {
                    error_message.set(Some(format!("读取配置文件失败: {}", e)));
                }
            }
            
            is_loading.set(false);
        }
    });
    
    // 保存配置文件的协程
    let save_config_coroutine = use_coroutine(move |mut rx: UnboundedReceiver<()>| async move {
        while let Some(_) = rx.next().await {
            is_loading.set(true);
            error_message.set(None);
            
            let path = config_path.read().clone();
            let content = config_content.read().clone();
            
            match tokio::fs::write(&path, &content).await {
                Ok(_) => {
                    save_message.set(Some("配置已保存".to_string()));
                    // 3秒后清除消息
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    save_message.set(None);
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
            error_message.set(None);
            
            // 创建默认配置内容
            let default_config = r#"# TagBox Configuration File
# TagBox 配置文件

[database]
# Database file path / 数据库文件路径
path = "tagbox_data/tagbox.db"

[import]
# Default import paths / 默认导入路径
default_paths = ["~/Documents", "~/Downloads"]

# Maximum file size in MB / 最大文件大小（MB）
max_file_size = 100

# Supported file types / 支持的文件类型
file_types = ["pdf", "epub", "txt", "md", "djvu", "mobi"]

[search]
# Maximum search results / 最大搜索结果数
max_results = 100

# Enable fuzzy search / 启用模糊搜索
fuzzy_search = true

# Search history limit / 搜索历史限制
history_limit = 50

[ui]
# Theme (light/dark) / 主题
theme = "light"

# Language / 语言
language = "zh-CN"

# Show status bar / 显示状态栏
show_status_bar = true
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
    
    // 文件选择协程
    let file_picker_coroutine = use_coroutine(move |mut rx: UnboundedReceiver<()>| async move {
        while let Some(_) = rx.next().await {
            if let Some(file) = rfd::AsyncFileDialog::new()
                .add_filter("TOML files", &["toml"])
                .add_filter("All files", &["*"])
                .set_directory(dirs::config_dir().unwrap_or_default())
                .pick_file()
                .await
            {
                let path = file.path().to_string_lossy().to_string();
                tracing::info!("Selected config file: {}", path);
                
                // 加载选择的配置文件
                config_path.set(path.clone());
                load_config_coroutine.send(path);
            }
        }
    });
    
    // 页面加载时自动加载配置
    use_effect(move || {
        load_config_coroutine.send(config_path.read().clone());
    });
    
    rsx! {
        rect {
            width: "100%",
            height: "100%",
            direction: "column",
            background: "rgb(245, 245, 245)",
            
            // 顶部栏
            rect {
                width: "100%",
                height: "60",
                background: "white",
                shadow: "0 2 4 0 rgb(200, 200, 200)",
                padding: "0 40",
                direction: "horizontal",
                content: "center space",
                
                rect {
                    direction: "horizontal",
                    spacing: "20",
                    cross_align: "center",
                    
                    label {
                        font_size: "24",
                        font_weight: "bold",
                        "Settings"
                    }
                }
                
                CustomButton {
                    text: "← Back",
                    variant: "secondary",
                    onpress: move |_| {
                        route.set(Route::Main);
                    },
                }
            }
            
            // 主内容区域
            ScrollView {
                rect {
                    width: "100%",
                    padding: "40",
                    direction: "column",
                    spacing: "30",
                    
                    // 消息显示区域
                    if let Some(message) = save_message.read().as_ref() {
                        rect {
                            width: "100%",
                            padding: "15",
                            background: "rgb(240, 255, 240)",
                            corner_radius: "8",
                            border: "1 solid rgb(100, 200, 100)",
                            
                            label {
                                color: "rgb(50, 150, 50)",
                                font_size: "14",
                                "{message}"
                            }
                        }
                    }
                    
                    if let Some(error) = error_message.read().as_ref() {
                        rect {
                            width: "100%",
                            padding: "15",
                            background: "rgb(255, 240, 240)",
                            corner_radius: "8",
                            border: "1 solid rgb(200, 100, 100)",
                            
                            label {
                                color: "rgb(200, 50, 50)",
                                font_size: "14",
                                "{error}"
                            }
                        }
                    }
                    
                    // 配置文件选择区域
                    rect {
                        width: "100%",
                        background: "white",
                        corner_radius: "8",
                        padding: "30",
                        shadow: "0 2 8 0 rgb(220, 220, 220)",
                        direction: "column",
                        spacing: "20",
                        
                        label {
                            font_size: "18",
                            font_weight: "bold",
                            color: "rgb(50, 50, 50)",
                            "Configuration File"
                        }
                        
                        rect {
                            direction: "horizontal",
                            spacing: "15",
                            cross_align: "center",
                            
                            rect {
                                width: "fill",
                                height: "40",
                                background: "rgb(245, 245, 245)",
                                corner_radius: "6",
                                padding: "0 15",
                                
                                Input {
                                    width: "100%",
                                    value: "{config_path}",
                                    onchange: move |e: String| {
                                        config_path.set(e);
                                    },
                                }
                            }
                            
                            CustomButton {
                                text: "Browse",
                                variant: "secondary",
                                onpress: move |_| {
                                    file_picker_coroutine.send(());
                                },
                            }
                            
                            CustomButton {
                                text: "New",
                                variant: "secondary",
                                onpress: move |_| {
                                    let new_path = format!("{}/tagbox.toml", 
                                        std::env::current_dir().unwrap().display());
                                    init_config_coroutine.send(new_path);
                                },
                            }
                        }
                    }
                    
                    // 配置内容编辑区域
                    rect {
                        width: "100%",
                        background: "white",
                        corner_radius: "8",
                        padding: "30",
                        shadow: "0 2 8 0 rgb(220, 220, 220)",
                        direction: "column",
                        spacing: "20",
                        
                        label {
                            font_size: "18",
                            font_weight: "bold",
                            color: "rgb(50, 50, 50)",
                            "Configuration Content"
                        }
                        
                        rect {
                            width: "100%",
                            height: "400",
                            background: "rgb(245, 245, 245)",
                            corner_radius: "6",
                            padding: "15",
                            border: "1 solid rgb(220, 220, 220)",
                            
                            ScrollView {
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
                            spacing: "15",
                            content: "center end",
                            
                            CustomButton {
                                text: "Reset",
                                variant: "secondary",
                                onpress: move |_| {
                                    // 重新加载文件
                                    let path = config_path.read().clone();
                                    load_config_coroutine.send(path);
                                },
                            }
                            
                            CustomButton {
                                text: if is_loading() { "Saving..." } else { "Save" },
                                variant: "primary",
                                disabled: is_loading(),
                                onpress: move |_| {
                                    save_config_coroutine.send(());
                                },
                            }
                        }
                    }
                    
                    // 数据库管理区域
                    rect {
                        width: "100%",
                        background: "white",
                        corner_radius: "8",
                        padding: "30",
                        shadow: "0 2 8 0 rgb(220, 220, 220)",
                        direction: "column",
                        spacing: "20",
                        
                        label {
                            font_size: "18",
                            font_weight: "bold",
                            color: "rgb(50, 50, 50)",
                            "Database Management"
                        }
                        
                        rect {
                            direction: "horizontal",
                            spacing: "15",
                            
                            CustomButton {
                                text: "Rebuild Index",
                                variant: "secondary",
                                onpress: move |_| {
                                    tracing::info!("Rebuild search index");
                                    // TODO: 实现索引重建
                                },
                            }
                            
                            CustomButton {
                                text: "Export Data",
                                variant: "secondary",
                                onpress: move |_| {
                                    tracing::info!("Export database");
                                    // TODO: 实现数据导出
                                },
                            }
                            
                            CustomButton {
                                text: "Backup",
                                variant: "secondary",
                                onpress: move |_| {
                                    tracing::info!("Backup database");
                                    // TODO: 实现数据库备份
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}