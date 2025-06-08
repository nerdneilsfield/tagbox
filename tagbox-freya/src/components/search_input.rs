use freya::prelude::*;
use crate::state::AppState;

pub fn SearchInput() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let mut input_value = use_signal(|| String::new());
    
    let _onsubmit = move || {
        let query = input_value.read().clone();
        app_state.write().search_query = query.clone();
        // TODO: 执行搜索
        tracing::info!("Searching for: {}", query);
    };
    
    rsx! {
        rect {
            width: "fill",
            height: "40",
            background: "rgb(245, 245, 245)",
            corner_radius: "20",
            padding: "0 15",
            
            Input {
                placeholder: "Search (e.g. tag:Rust -tag:旧版)",
                value: "{input_value}",
                onchange: move |e: String| {
                    input_value.set(e);
                },
            }
        }
    }
}