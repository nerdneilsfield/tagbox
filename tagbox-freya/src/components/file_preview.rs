use freya::prelude::*;
use crate::state::AppState;
use crate::router::Route;
use crate::utils::clipboard::copy_to_clipboard;

pub fn FilePreview() -> Element {
    let app_state = use_context::<Signal<Option<AppState>>>();
    let mut route = use_context::<Signal<Route>>();
    let selected_file = app_state.read().as_ref()
        .and_then(|s| s.selected_file.clone());
    
    match selected_file {
        Some(file) => {
            let _file_path = file.path.clone();
            let file_path_copy = file.path.clone();
            let file_path_open = file.path.clone();
            let file_path3 = file.path.clone();
            let file_path4 = file.path.clone();
            let file_id = file.id.clone();
            
            rsx! {
            ScrollView {
                rect {
                    width: "100%",
                    direction: "column",
                    spacing: "20",
                    
                    // 标题
                    label {
                        font_size: "24",
                        font_weight: "bold",
                        color: "rgb(30, 30, 30)",
                        "{file.title}"
                    }
                    
                    // 文件路径
                    rect {
                        direction: "column",
                        spacing: "5",
                        
                        label {
                            font_size: "12",
                            color: "rgb(120, 120, 120)",
                            "Path:"
                        }
                        
                        rect {
                            padding: "8",
                            background: "rgb(245, 245, 245)",
                            corner_radius: "4",
                            onclick: move |_| {
                                if let Err(e) = copy_to_clipboard(&file_path_copy) {
                                    tracing::error!("Failed to copy: {}", e);
                                }
                            },
                            
                            label {
                                font_size: "13",
                                color: "rgb(80, 80, 80)",
                                "{file.path}"
                            }
                        }
                    }
                    
                    // 作者
                    if !file.authors.is_empty() {
                        rect {
                            direction: "column",
                            spacing: "5",
                            
                            label {
                                font_size: "12",
                                color: "rgb(120, 120, 120)",
                                "Authors:"
                            }
                            
                            rect {
                                direction: "horizontal",
                                spacing: "8",
                                
                                for author in &file.authors {
                                    rect {
                                        padding: "4 8",
                                        background: "rgb(220, 220, 220)",
                                        corner_radius: "12",
                                        
                                        label {
                                            font_size: "12",
                                            "{author}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // 标签
                    if !file.tags.is_empty() {
                        rect {
                            direction: "column",
                            spacing: "5",
                            
                            label {
                                font_size: "12",
                                color: "rgb(120, 120, 120)",
                                "Tags:"
                            }
                            
                            rect {
                                direction: "horizontal",
                                spacing: "8",
                                
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
                        }
                    }
                    
                    // 摘要
                    if let Some(summary) = &file.summary {
                        rect {
                            direction: "column",
                            spacing: "5",
                            
                            label {
                                font_size: "12",
                                color: "rgb(120, 120, 120)",
                                "Summary:"
                            }
                            
                            rect {
                                padding: "12",
                                background: "rgb(250, 250, 250)",
                                corner_radius: "4",
                                
                                label {
                                    font_size: "14",
                                    line_height: "1.5",
                                    "{summary}"
                                }
                            }
                        }
                    }
                    
                    // 文件信息
                    rect {
                        direction: "column",
                        spacing: "5",
                        
                        label {
                            font_size: "12",
                            color: "rgb(120, 120, 120)",
                            "File Info:"
                        }
                        
                        rect {
                            direction: "column",
                            spacing: "3",
                            
                            label {
                                font_size: "12",
                                color: "rgb(100, 100, 100)",
                                "Size: {format_file_size(file.size)}"
                            }
                            
                            label {
                                font_size: "12",
                                color: "rgb(100, 100, 100)",
                                "Modified: {file.modified_at}"
                            }
                            
                            label {
                                font_size: "12",
                                color: "rgb(100, 100, 100)",
                                "Imported: {file.imported_at}"
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
                                // TODO: 打开文件
                                tracing::info!("Open file: {}", file_path_open);
                            },
                            
                            label { "Open File" }
                        }
                        
                        Button {
                            onpress: move |_| {
                                route.set(Route::Edit(file_id.clone()));
                            },
                            
                            label { "Edit" }
                        }
                        
                        Button {
                            onpress: move |_| {
                                // TODO: 复制文件夹路径
                                tracing::info!("CD to: {}", file_path3.clone());
                            },
                            
                            label { "CD" }
                        }
                        
                        Button {
                            onpress: move |_| {
                                if let Err(e) = copy_to_clipboard(&file_path4) {
                                    tracing::error!("Failed to copy: {}", e);
                                }
                            },
                            
                            label { "Copy Path" }
                        }
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
                    font_size: "16",
                    color: "rgb(150, 150, 150)",
                    "Select a file to preview"
                }
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