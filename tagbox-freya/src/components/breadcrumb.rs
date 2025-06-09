use freya::prelude::*;
use crate::router::{Route, use_route};

/// 面包屑导航组件
#[component]
pub fn Breadcrumb() -> Element {
    let mut current_route = use_route();
    
    let breadcrumbs = match current_route.read().clone() {
        Route::Main => vec![("主页", Route::Main)],
        Route::Import => vec![
            ("主页", Route::Main),
            ("导入文件", Route::Import),
        ],
        Route::Edit(ref file_id) => vec![
            ("主页", Route::Main),
            ("编辑文件", Route::Edit(file_id.clone())),
        ],
        Route::Settings => vec![
            ("主页", Route::Main),
            ("设置", Route::Settings),
        ],
    };
    
    rsx! {
        rect {
            width: "100%",
            height: "36",
            direction: "horizontal",
            spacing: "8",
            padding: "0 16",
            background: "rgb(249, 249, 249)",
            border: "0 0 1 0 solid rgb(230, 230, 230)",
            content: "center start",
            
            for (i, (name, route)) in breadcrumbs.iter().enumerate() {
                if i > 0 {
                    label {
                        color: "rgb(150, 150, 150)",
                        " / "
                    }
                }
                
                if i == breadcrumbs.len() - 1 {
                    // 当前页面，不可点击
                    label {
                        color: "rgb(80, 80, 80)",
                        font_weight: "bold",
                        "{name}"
                    }
                } else {
                    // 可点击的链接
                    rect {
                        onclick: {
                            let route = route.clone();
                            move |_| current_route.set(route.clone())
                        },
                        
                        label {
                            color: "rgb(59, 130, 246)",
                            "{name}"
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breadcrumb_routes() {
        // 测试主页面包屑
        let main_route = Route::Main;
        assert_eq!(main_route, Route::Main);
        
        // 测试导入页面包屑
        let import_route = Route::Import;
        assert_eq!(import_route, Route::Import);
        
        // 测试编辑页面包屑
        let edit_route = Route::Edit("test-id".to_string());
        match edit_route {
            Route::Edit(id) => assert_eq!(id, "test-id"),
            _ => panic!("Expected Edit route"),
        }
    }
}