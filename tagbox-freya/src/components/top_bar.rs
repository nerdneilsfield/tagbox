use freya::prelude::*;
use crate::components::{SearchInput, AdvancedSearchModal};
use crate::router::Route;

pub fn TopBar() -> Element {
    let mut show_advanced_search = use_signal(|| false);
    let mut route = use_context::<Signal<Route>>();
    
    rsx! {
        rect {
            width: "100%",
            height: "60",
            background: "white",
            padding: "10 20",
            direction: "horizontal",
            
            // Logo和标题
            rect {
                width: "auto",
                height: "100%",
                content: "center",
                margin: "0 20 0 0",
                
                label {
                    font_size: "24",
                    font_weight: "bold",
                    color: "rgb(50, 50, 50)",
                    "TagBox"
                }
            }
            
            // 搜索区域
            rect {
                width: "flex",
                height: "100%",
                direction: "horizontal",
                spacing: "10",
                content: "center",
                
                SearchInput {}
                
                Button {
                    onpress: move |_| show_advanced_search.set(true),
                    
                    label { "Advanced" }
                }
            }
            
            // 导入按钮
            rect {
                width: "auto",
                height: "100%",
                content: "center",
                margin: "0 0 0 20",
                
                Button {
                    onpress: move |_| {
                        route.set(Route::Import);
                    },
                    
                    label { "Import File" }
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