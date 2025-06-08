use freya::prelude::*;
use crate::state::{AppState, Category, FileEntry};

pub fn CategoryTree() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let categories = app_state.read().categories.clone();
    
    rsx! {
        ScrollView {
            rect {
                width: "100%",
                direction: "column",
                
                label {
                    font_size: "18",
                    font_weight: "bold",
                    margin: "0 0 15 0",
                    "Categories"
                }
                
                for category in categories {
                    CategoryNode {
                        key: "{category.id}",
                        category: category.clone(),
                        level: 0
                    }
                }
            }
        }
    }
}

#[component]
fn CategoryNode(category: Category, level: u8) -> Element {
    let mut expanded = use_signal(|| false);
    let has_children = !category.children.is_empty();
    let has_files = !category.files.is_empty();
    let indent = level * 20;
    
    rsx! {
        rect {
            width: "100%",
            direction: "column",
            
            // 分类节点
            rect {
                width: "100%",
                padding: "8 10 8 {indent + 10}",
                direction: "horizontal",
                content: "center start",
                spacing: "8",
                onclick: move |_| {
                    if has_children || has_files {
                        expanded.toggle();
                    }
                },
                
                // 展开/折叠图标
                if has_children || has_files {
                    label {
                        font_size: "12",
                        color: "rgb(150, 150, 150)",
                        width: "16",
                        if expanded() { "▼" } else { "▶" }
                    }
                } else {
                    rect { width: "16" }
                }
                
                // 分类名称
                label {
                    font_size: "14",
                    color: "rgb(50, 50, 50)",
                    "{category.name}"
                }
                
                // 文件数量
                if has_files {
                    label {
                        font_size: "12",
                        color: "rgb(150, 150, 150)",
                        "({category.files.len()})"
                    }
                }
            }
            
            // 展开的内容
            if expanded() {
                rect {
                    width: "100%",
                    direction: "column",
                    
                    // 子分类
                    for child in &category.children {
                        CategoryNode {
                            key: "{child.id}",
                            category: child.clone(),
                            level: level + 1
                        }
                    }
                    
                    // 文件列表
                    for file in &category.files {
                        FileNode {
                            key: "{file.id}",
                            file: file.clone(),
                            level: level + 1
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn FileNode(file: FileEntry, level: u8) -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let indent = (level + 1) * 20;
    let is_selected = app_state.read().selected_file.as_ref() == Some(&file);
    
    rsx! {
        rect {
            width: "100%",
            padding: "6 10 6 {indent + 10}",
            background: if is_selected { "rgb(240, 240, 255)" } else { "transparent" },
            onclick: move |_| {
                app_state.write().selected_file = Some(file.clone());
            },
            
            label {
                font_size: "13",
                color: if is_selected { "rgb(80, 80, 255)" } else { "rgb(80, 80, 80)" },
                max_lines: "1",
                text_overflow: "ellipsis",
                "{file.title}"
            }
        }
    }
}