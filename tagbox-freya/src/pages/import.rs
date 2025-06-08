use freya::prelude::*;
use crate::state::AppState;
use crate::components::{DragDropArea, SelectedFileDisplay};
use std::path::PathBuf;

pub fn ImportPage() -> Element {
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
    
    let file_selected = selected_file.read().is_some();
    
    rsx! {
        ScrollView {
            rect {
                width: "100%",
                padding: "40",
                direction: "column",
                spacing: "30",
                
                // 页面标题
                label {
                    font_size: "28",
                    font_weight: "bold",
                    "Import New File"
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
                                    // TODO: 提取元数据
                                    tracing::info!("Extract metadata");
                                },
                                
                                label { "Extract Metadata" }
                            }
                            
                            Button {
                                onpress: move |_| {
                                    // TODO: 导入并移动文件
                                    tracing::info!("Import and move file");
                                },
                                
                                label { "Import and Move" }
                            }
                            
                            Button {
                                onpress: move |_| {
                                    // TODO: 导入但保留原文件
                                    tracing::info!("Import and keep original");
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