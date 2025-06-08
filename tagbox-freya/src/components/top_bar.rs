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
            shadow: "0 2 4 0 rgb(0, 0, 0, 10)",
            padding: "10 20",
            direction: "horizontal",
            main_align: "center",
            cross_align: "center",
            
            // Logo和标题
            rect {
                direction: "horizontal",
                cross_align: "center",
                spacing: "10",
                
                label {
                    font_size: "24",
                    font_weight: "bold",
                    color: "rgb(50, 50, 50)",
                    "TagBox"
                }
            }
            
            // 搜索区域
            rect {
                width: "fill",
                max_width: "600",
                margin: "0 20",
                direction: "horizontal",
                spacing: "10",
                
                SearchInput {}
                
                Button {
                    onpress: move |_| show_advanced_search.set(true),
                    
                    label { "Advanced" }
                }
            }
            
            // 导入按钮
            Button {
                onpress: move |_| {
                    route.set(Route::Import);
                },
                
                label { "Import File" }
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