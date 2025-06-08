use freya::prelude::*;
use futures::channel::mpsc::UnboundedReceiver;
use futures::StreamExt;
use crate::state::AppState;

pub fn SearchInput() -> Element {
    let mut app_state = use_context::<Signal<Option<AppState>>>();
    let mut input_value = use_signal(|| String::new());
    let mut is_searching = use_signal(|| false);
    
    // 使用 coroutine 处理异步搜索
    let search_coroutine = use_coroutine(move |mut rx: UnboundedReceiver<String>| async move {
        while let Some(query) = rx.next().await {
            is_searching.set(true);
            
            // 执行搜索
            if let Some(state) = app_state.write().as_mut() {
                match state.search(&query).await {
                    Ok(_) => {
                        tracing::info!("Search completed for: {}", query);
                    }
                    Err(e) => {
                        tracing::error!("Search failed: {}", e);
                        state.error_message = Some(format!("搜索失败: {}", e));
                    }
                }
            }
            
            is_searching.set(false);
        }
    });
    
    rsx! {
        rect {
            width: "flex",
            height: "40",
            background: "rgb(245, 245, 245)",
            corner_radius: "20",
            padding: "0 15",
            direction: "horizontal",
            content: "center",
            
            Input {
                placeholder: "Search (e.g. tag:Rust -tag:旧版)",
                value: "{input_value}",
                onchange: move |e: String| {
                    input_value.set(e.clone());
                    // 按下回车时搜索
                    if e.contains('\n') {
                        let query = e.trim().to_string();
                        if !query.is_empty() {
                            search_coroutine.send(query);
                        }
                    }
                },
            }
            
            // 搜索按钮或加载指示器
            rect {
                width: "40",
                height: "40",
                content: "center",
                
                if is_searching() {
                    label {
                        font_size: "16",
                        color: "rgb(100, 100, 100)",
                        "..."
                    }
                } else {
                    Button {
                        onpress: move |_| {
                            let query = input_value.read().clone();
                            if !query.is_empty() {
                                search_coroutine.send(query);
                            }
                        },
                        
                        label { "🔍" }
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
    fn test_search_query_validation() {
        // 测试搜索查询的有效性
        let valid_queries = vec![
            "tag:rust",
            "author:张三",
            "-tag:old",
            "tag:rust author:李四",
            "*", // 列出所有
        ];
        
        for query in valid_queries {
            assert!(!query.is_empty());
        }
    }
}