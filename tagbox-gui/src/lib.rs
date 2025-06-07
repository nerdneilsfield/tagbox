pub mod app;
pub mod components;
pub mod state;
pub mod themes;
pub mod utils;

use app::App;
use tagbox_core::config::AppConfig;

/// 运行 TagBox GUI 应用
pub async fn run_app(config: AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    // 创建并运行应用
    let mut app = App::new(config)?;
    app.run()
}