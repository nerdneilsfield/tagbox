use freya::prelude::*;
use crate::state::{AppState, Category, FileEntry};

#[component]
fn ShowAllButton(total: usize) -> Element {
    let mut app_state = use_context::<Signal<Option<AppState>>>();
    let is_selected = app_state.read().as_ref()
        .map(|s| s.selected_category.is_none())
        .unwrap_or(true);
    
    rsx! {
        rect {
            width: "100%",
            padding: "8 10",
            background: if is_selected { "rgb(240, 240, 255)" } else { "transparent" },
            corner_radius: "4",
            margin: "0 0 5 0",
            onclick: move |_| {
                if let Some(state) = app_state.write().as_mut() {
                    state.selected_category = None;
                }
            },
            
            rect {
                direction: "horizontal",
                spacing: "8",
                content: "center start",
                
                label {
                    font_size: "12",
                    color: "rgb(150, 150, 150)",
                    width: "16",
                    "📁"
                }
                
                label {
                    font_size: "14",
                    color: if is_selected { "rgb(80, 80, 255)" } else { "rgb(50, 50, 50)" },
                    font_weight: if is_selected { "bold" } else { "normal" },
                    "显示全部"
                }
                
                label {
                    font_size: "12",
                    color: "rgb(150, 150, 150)",
                    "({total})"
                }
            }
        }
    }
}

pub fn CategoryTree() -> Element {
    let app_state = use_context::<Signal<Option<AppState>>>();
    
    let (categories, total) = match app_state.read().as_ref() {
        Some(state) => (state.categories.clone(), state.search_results.total_count),
        None => (vec![], 0)
    };
    
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
                
                // "显示全部"选项
                ShowAllButton {
                    total: total
                }
                
                // 分类列表
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
    let mut app_state = use_context::<Signal<Option<AppState>>>();
    let mut expanded = use_signal(|| false);
    let has_children = !category.children.is_empty();
    let has_files = !category.files.is_empty();
    let indent = level * 20;
    
    let is_selected = app_state.read().as_ref()
        .and_then(|s| s.selected_category.as_ref())
        .map(|c| c == &category.id)
        .unwrap_or(false);
    
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
                background: if is_selected { "rgb(240, 240, 255)" } else { "transparent" },
                corner_radius: "4",
                onclick: move |_| {
                    if has_children || has_files {
                        // 如果有子项，点击展开/折叠
                        expanded.toggle();
                    }
                    
                    // 同时选择分类
                    if let Some(state) = app_state.write().as_mut() {
                        if state.selected_category == Some(category.id.clone()) {
                            // 如果已选中，则取消选择
                            state.selected_category = None;
                        } else {
                            // 否则选择这个分类
                            state.selected_category = Some(category.id.clone());
                        }
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
                    color: if is_selected { "rgb(80, 80, 255)" } else { "rgb(50, 50, 50)" },
                    font_weight: if is_selected { "bold" } else { "normal" },
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
    let mut app_state = use_context::<Signal<Option<AppState>>>();
    let indent = (level + 1) * 20;
    let is_selected = app_state.read().as_ref()
        .and_then(|s| s.selected_file.as_ref())
        .map(|f| f.id == file.id)
        .unwrap_or(false);
    
    rsx! {
        rect {
            width: "100%",
            padding: "6 10 6 {indent + 10}",
            background: if is_selected { "rgb(240, 240, 255)" } else { "transparent" },
            onclick: move |_| {
                if let Some(state) = app_state.write().as_mut() {
                    state.selected_file = Some(file.clone());
                }
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