#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use freya::prelude::*;

mod app;
mod components;
mod pages;
mod router;
mod state;
mod utils;

use app::App;

fn main() {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 启动应用
    launch_cfg(
        App,
        LaunchConfig::<()>::new()
            .with_title("TagBox")
            .with_size(1280.0, 800.0)
    );
}