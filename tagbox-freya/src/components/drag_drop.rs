use freya::prelude::*;
use std::path::PathBuf;
use crate::components::CustomButton;

#[component]
pub fn DragDropArea(
    onfile: EventHandler<PathBuf>,
) -> Element {
    let mut is_dragging = use_signal(|| false);
    
    rsx! {
        rect {
            width: "100%",
            height: "200",
            background: if is_dragging() { "rgb(230, 230, 255)" } else { "rgb(245, 245, 245)" },
            corner_radius: "8",
            border: if is_dragging() { "2 solid rgb(100, 100, 255)" } else { "2 dashed rgb(200, 200, 200)" },
            content: "center",
            onclick: move |_| {
                // TODO: 打开文件选择对话框
                // 在实际实现中，这里需要集成系统文件对话框
                tracing::info!("Open file dialog");
            },
            onmouseenter: move |_| {
                // 检测拖放状态
            },
            onmouseleave: move |_| {
                is_dragging.set(false);
            },
            
            rect {
                direction: "column",
                spacing: "15",
                content: "center",
                
                // 图标
                rect {
                    width: "60",
                    height: "60",
                    content: "center",
                    
                    label {
                        font_size: "40",
                        color: if is_dragging() { "rgb(100, 100, 255)" } else { "rgb(180, 180, 180)" },
                        "📁"
                    }
                }
                
                label {
                    font_size: "16",
                    color: if is_dragging() { "rgb(80, 80, 200)" } else { "rgb(150, 150, 150)" },
                    font_weight: if is_dragging() { "bold" } else { "normal" },
                    "Drag and drop file here"
                }
                
                label {
                    font_size: "14",
                    color: "rgb(180, 180, 180)",
                    "or click to browse"
                }
                
                // 支持的文件类型提示
                label {
                    font_size: "12",
                    color: "rgb(200, 200, 200)",
                    margin: "10 0 0 0",
                    "Supports: PDF, EPUB, TXT, JSON"
                }
            }
        }
    }
}

/// 文件选择按钮
#[component]
pub fn FileSelectButton(
    onfile: EventHandler<PathBuf>,
) -> Element {
    rsx! {
        CustomButton {
            text: "Browse Files",
            variant: "secondary",
            onpress: move |_| {
                // TODO: 打开文件选择对话框
                tracing::info!("Select file");
            },
        }
    }
}

/// 显示已选择的文件
#[component]
pub fn SelectedFileDisplay(
    file_path: Option<PathBuf>,
    onremove: EventHandler<()>,
) -> Element {
    match file_path {
        Some(path) => {
            let filename = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown file");
            let file_size = std::fs::metadata(&path)
                .map(|m| format_file_size(m.len()))
                .unwrap_or_else(|_| "Unknown size".to_string());
            
            rsx! {
                rect {
                    width: "100%",
                    padding: "15",
                    background: "rgb(250, 250, 255)",
                    corner_radius: "6",
                    direction: "horizontal",
                    content: "center start",
                    
                    // 文件图标
                    label {
                        font_size: "24",
                        margin: "0 10 0 0",
                        "📄"
                    }
                    
                    // 文件信息
                    rect {
                        width: "flex",
                        direction: "column",
                        spacing: "5",
                        
                        label {
                            font_size: "14",
                            font_weight: "bold",
                            color: "rgb(50, 50, 50)",
                            "{filename}"
                        }
                        
                        label {
                            font_size: "12",
                            color: "rgb(120, 120, 120)",
                            "{file_size}"
                        }
                    }
                    
                    // 移除按钮
                    rect {
                        width: "30",
                        height: "30",
                        content: "center",
                        corner_radius: "15",
                        background: "rgb(255, 100, 100)",
                        onclick: move |_| onremove.call(()),
                        
                        label {
                            color: "white",
                            font_size: "16",
                            "×"
                        }
                    }
                }
            }
        },
        None => rsx! { rect {} }
    }
}

fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    format!("{:.2} {}", size, UNITS[unit_index])
}