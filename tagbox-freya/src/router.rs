use freya::prelude::*;
use crate::pages::{ImportPage, EditPage};
use crate::app::MainView;

#[derive(Clone, Debug, PartialEq)]
pub enum Route {
    Main,
    Import,
    Edit(String),
}

pub fn Router() -> Element {
    let current_route = use_context_provider(|| Signal::new(Route::Main));
    
    let route = current_route.read().clone();
    match route {
        Route::Main => rsx! { MainView {} },
        Route::Import => rsx! { ImportPage {} },
        Route::Edit(file_id) => rsx! { EditPage { file_id } },
    }
}

/// 导航到指定路由
pub fn navigate_to(_route: Route) {
    // 这个函数需要在组件内部调用，以获取路由上下文
    // 在实际使用时，通过 use_context 获取路由信号并更新
}