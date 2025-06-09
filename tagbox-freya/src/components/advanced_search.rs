use freya::prelude::*;
use crate::components::CustomButton;

#[component]
pub fn AdvancedSearchModal(onclose: EventHandler<()>) -> Element {
    let tag_input = use_signal(|| String::new());
    let title_input = use_signal(|| String::new());
    let author_input = use_signal(|| String::new());
    let mut year_start = use_signal(|| String::new());
    let mut year_end = use_signal(|| String::new());
    
    let onsearch = move || {
        // TODO: 构建搜索查询
        let query = format!(
            "tag:{} title:{} author:{}",
            tag_input.read(),
            title_input.read(),
            author_input.read()
        );
        tracing::info!("Advanced search: {}", query);
        onclose.call(());
    };
    
    rsx! {
        // 模态框背景 - 使用带透明度的容器
        rect {
            position: "absolute",
            width: "100%",
            height: "100%",
            main_align: "center",
            cross_align: "center",
            
            // 背景层
            rect {
                position: "absolute",
                width: "100%",
                height: "100%",
                background: "rgb(50, 50, 50)",
                opacity: "0.8",
                onclick: move |_| onclose.call(()),
            }
            
            // 模态框内容
            rect {
                width: "600",
                max_height: "80%",
                background: "white",
                corner_radius: "8",
                shadow: "0 4 20 0 rgb(200, 200, 200)",
                padding: "30",
                onclick: move |e| e.stop_propagation(),
                
                ScrollView {
                    rect {
                        width: "100%",
                        direction: "column",
                        spacing: "20",
                        
                        // 标题
                        rect {
                            direction: "horizontal",
                            cross_align: "center",
                            
                            label {
                                font_size: "20",
                                font_weight: "bold",
                                width: "fill",
                                "Advanced Search"
                            }
                            
                            rect {
                                width: "30",
                                height: "30",
                                main_align: "center",
                                cross_align: "center",
                                onclick: move |_| onclose.call(()),
                                
                                label {
                                    font_size: "20",
                                    color: "rgb(150, 150, 150)",
                                    "×"
                                }
                            }
                        }
                        
                        // 标签输入
                        SearchField {
                            label: "Tags",
                            placeholder: "Enter tags separated by commas",
                            value: tag_input,
                        }
                        
                        // 标题输入
                        SearchField {
                            label: "Title contains",
                            placeholder: "Enter title keywords",
                            value: title_input,
                        }
                        
                        // 作者输入
                        SearchField {
                            label: "Author",
                            placeholder: "Enter author name",
                            value: author_input,
                        }
                        
                        // 分类选择
                        rect {
                            direction: "column",
                            spacing: "8",
                            
                            label {
                                font_size: "14",
                                font_weight: "bold",
                                color: "rgb(70, 70, 70)",
                                "Category"
                            }
                            
                            rect {
                                direction: "horizontal",
                                spacing: "10",
                                
                                // TODO: 实现级联选择器
                                rect {
                                    width: "180",
                                    height: "40",
                                    padding: "10",
                                    background: "rgb(245, 245, 245)",
                                    corner_radius: "4",
                                    
                                    label {
                                        color: "rgb(150, 150, 150)",
                                        "Select Level 1"
                                    }
                                }
                                
                                rect {
                                    width: "180",
                                    height: "40",
                                    padding: "10",
                                    background: "rgb(245, 245, 245)",
                                    corner_radius: "4",
                                    
                                    label {
                                        color: "rgb(150, 150, 150)",
                                        "Select Level 2"
                                    }
                                }
                                
                                rect {
                                    width: "180",
                                    height: "40",
                                    padding: "10",
                                    background: "rgb(245, 245, 245)",
                                    corner_radius: "4",
                                    
                                    label {
                                        color: "rgb(150, 150, 150)",
                                        "Select Level 3"
                                    }
                                }
                            }
                        }
                        
                        // 年份范围
                        rect {
                            direction: "column",
                            spacing: "8",
                            
                            label {
                                font_size: "14",
                                font_weight: "bold",
                                color: "rgb(70, 70, 70)",
                                "Year Range"
                            }
                            
                            rect {
                                direction: "horizontal",
                                spacing: "10",
                                cross_align: "center",
                                
                                Input {
                                    width: "100",
                                    placeholder: "Start",
                                    value: "{year_start}",
                                    onchange: move |e: String| {
                                        year_start.set(e);
                                    },
                                }
                                
                                label {
                                    font_size: "14",
                                    color: "rgb(120, 120, 120)",
                                    "to"
                                }
                                
                                Input {
                                    width: "100",
                                    placeholder: "End",
                                    value: "{year_end}",
                                    onchange: move |e: String| {
                                        year_end.set(e);
                                    },
                                }
                            }
                        }
                        
                        // 搜索按钮
                        rect {
                            direction: "horizontal",
                            spacing: "10",
                            main_align: "end",
                            margin: "20 0 0 0",
                            
                            CustomButton {
                                text: "Cancel",
                                variant: "secondary",
                                onpress: move |_| onclose.call(()),
                            }
                            
                            CustomButton {
                                text: "Search",
                                variant: "primary",
                                onpress: move |_| onsearch(),
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SearchField(
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
                color: "rgb(70, 70, 70)",
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