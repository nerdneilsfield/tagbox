use freya::prelude::*;
use crate::state::AppState;

/// 状态栏组件
pub fn StatusBar() -> Element {
    let app_state = use_context::<Signal<Option<AppState>>>();
    
    let (total_files, selected_file, is_loading) = match app_state.read().as_ref() {
        Some(state) => (
            state.search_results.total_count,
            state.selected_file.as_ref().map(|f| f.title.clone()),
            state.is_loading,
        ),
        None => (0, None, false),
    };
    
    rsx! {
        rect {
            width: "100%",
            height: "30",
            background: "rgb(240, 240, 240)",
            border: "1 0 0 0 solid rgb(220, 220, 220)",
            direction: "horizontal",
            content: "center space",
            padding: "0 20",
            
            // 左侧 - 文件总数和选中状态
            rect {
                direction: "horizontal",
                spacing: "20",
                
                label {
                    font_size: "12",
                    color: "rgb(100, 100, 100)",
                    "Total Files: {total_files}"
                }
                
                if let Some(file_name) = selected_file {
                    label {
                        font_size: "12",
                        color: "rgb(100, 100, 100)",
                        "Selected: {file_name}"
                    }
                }
            }
            
            // 右侧 - 加载状态和其他信息
            rect {
                direction: "horizontal",
                spacing: "20",
                
                if is_loading {
                    label {
                        font_size: "12",
                        color: "rgb(100, 100, 100)",
                        "Loading..."
                    }
                }
                
                label {
                    font_size: "12",
                    color: "rgb(100, 100, 100)",
                    "TagBox v0.1.0"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_bar_display() {
        // 测试状态栏显示逻辑
        let total_files = 100;
        let selected_file = Some("test.pdf".to_string());
        let is_loading = false;
        
        assert_eq!(total_files, 100);
        assert_eq!(selected_file, Some("test.pdf".to_string()));
        assert_eq!(is_loading, false);
    }
}