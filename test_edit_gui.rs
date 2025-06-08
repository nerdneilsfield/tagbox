use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    println!("Testing TagBox GUI editing functionality...");
    
    // 加载配置
    let config_path = Path::new("config.toml");
    let config = match tagbox_core::load_config(config_path).await {
        Ok(config) => {
            println!("✅ Successfully loaded config");
            println!("Database path: {}", config.database.path.display());
            config
        },
        Err(e) => {
            println!("❌ Failed to load config: {}", e);
            let default_config = tagbox_core::config::AppConfig::default();
            println!("Using default config with database: {}", default_config.database.path.display());
            default_config
        }
    };
    
    // 启动 GUI 应用
    println!("Starting GUI application...");
    match tagbox_gui::run_app(config).await {
        Ok(_) => println!("✅ GUI application finished successfully"),
        Err(e) => println!("❌ GUI application error: {}", e),
    }
    
    Ok(())
}