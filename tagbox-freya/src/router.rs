use freya::prelude::*;
use std::rc::Rc;
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

/// 获取当前路由
pub fn use_route() -> Signal<Route> {
    use_context::<Signal<Route>>()
}

// 注意：这些函数已被 use_navigator() hook 替代
// 保留这些函数只是为了向后兼容