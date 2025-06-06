mod app;
mod state;
mod components;
mod utils;
mod themes;

use std::path::Path;
use tracing_subscriber;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    // 加载配置 - 这里使用默认配置路径，实际应用中可以从命令行参数获取
    let config_path = Path::new("./config.toml");
    
    // 由于我们需要在同步上下文中初始化异步代码，我们使用 tokio 的 Runtime
    let rt = tokio::runtime::Runtime::new()?;
    
    let config = rt.block_on(async {
        match tagbox_core::load_config(config_path).await {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to load config from {}: {}", config_path.display(), e);
                eprintln!("Using default configuration...");
                // 使用默认配置
                tagbox_core::config::AppConfig::default()
            }
        }
    });
    
    // 创建并运行应用
    let mut app = app::App::new(config)?;
    app.run()?;
    
    Ok(())
}
