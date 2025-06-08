use freya::prelude::*;
use crate::state::{AppState, FileEntry};

#[component]
pub fn EditPage(file_id: String) -> Element {
    let app_state = use_context::<Signal<Option<AppState>>>();
    
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
            let title = use_signal(|| file.title.clone());
            let authors = use_signal(|| file.authors.join(", "));
            let tags = use_signal(|| file.tags.join(", "));
            let mut summary = use_signal(|| file.summary.clone().unwrap_or_default());
            let category1 = use_signal(|| String::new());
            let category2 = use_signal(|| String::new());
            let category3 = use_signal(|| String::new());
            
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
                                    // TODO: 返回主页面
                                    tracing::info!("Navigate back");
                                },
                                
                                label { "← Back" }
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
                                    // TODO: 重新提取元数据
                                    tracing::info!("Re-extract metadata");
                                },
                                
                                label { "Re-extract Metadata" }
                            }
                            
                            Button {
                                onpress: move |_| {
                                    // TODO: 重置到原始值
                                    tracing::info!("Reset to original");
                                },
                                
                                label { "Reset to Original" }
                            }
                            
                            rect { width: "flex" }
                            
                            Button {
                                onpress: move |_| {
                                    // TODO: 删除文件
                                    tracing::info!("Delete file");
                                },
                                
                                label { "Delete" }
                            }
                            
                            Button {
                                onpress: move |_| {
                                    // TODO: 取消编辑
                                    tracing::info!("Cancel edit");
                                },
                                
                                label { "Cancel" }
                            }
                            
                            Button {
                                onpress: move |_| {
                                    // TODO: 保存更改
                                    tracing::info!("Save changes");
                                },
                                
                                label { "Save" }
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