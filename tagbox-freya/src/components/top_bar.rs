use freya::prelude::*;
use crate::components::{SearchInput, AdvancedSearchModal, CustomButton};
use crate::router::{Route, use_route};

pub fn TopBar() -> Element {
    let mut show_advanced_search = use_signal(|| false);
    let mut route = use_route();
    
    rsx! {
        rect {
            width: "100%",
            height: "56",
            background: "rgb(255, 255, 255)",
            shadow: "0 2 4 0 rgba(0, 0, 0, 0.1)",
            direction: "horizontal",
            padding: "0 16",
            content: "center start",
            position: "relative",
            
            // Logo区域 - 固定宽度
            rect {
                width: "150",
                height: "100%",
                content: "center start",
                direction: "horizontal",
                spacing: "12",
                
                label {
                    font_size: "20",
                    font_weight: "bold",
                    color: "rgb(30, 30, 30)",
                    "TagBox"
                }
            }
            
            // 搜索区域 - 中间部分
            rect {
                width: "500",
                max_width: "500",
                height: "100%",
                direction: "horizontal",
                spacing: "8",
                content: "center",
                margin: "0 16",
                
                rect {
                    width: "400",
                    height: "36",
                    
                    SearchInput {}
                }
                
                CustomButton {
                    text: "高级搜索",
                    variant: "secondary",
                    onpress: move |_| show_advanced_search.set(true),
                }
            }
            
            // 占位区域 - 推动按钮到右侧
            rect {
                width: "fill",
                height: "100%",
            }
            
            // 右侧按钮组 - 固定在右侧
            rect {
                width: "auto",
                height: "100%",
                direction: "horizontal",
                spacing: "8",
                content: "center end",
                
                CustomButton {
                    text: "导入文件",
                    variant: "primary",
                    onpress: move |_| {
                        route.set(Route::Import);
                    },
                }
                
                CustomButton {
                    text: "⚙ 设置",
                    variant: "secondary",
                    onpress: move |_| {
                        route.set(Route::Settings);
                    },
                }
            }
        }
        
        // 高级搜索模态框
        if show_advanced_search() {
            AdvancedSearchModal {
                onclose: move |_| show_advanced_search.set(false)
            }
        }
    }
}