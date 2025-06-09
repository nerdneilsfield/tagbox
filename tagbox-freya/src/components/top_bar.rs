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
            shadow: "0 2 4 0 rgb(200, 200, 200)",
            direction: "horizontal",
            padding: "0 16",
            content: "center space",
            
            // Logo
            label {
                font_size: "20",
                font_weight: "bold",
                color: "rgb(30, 30, 30)",
                "TagBox"
            }
            
            // 搜索区域
            rect {
                direction: "horizontal",
                spacing: "8",
                content: "center",
                
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
            
            // 右侧按钮组
            rect {
                direction: "horizontal",
                spacing: "8",
                
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