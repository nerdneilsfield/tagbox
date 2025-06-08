use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    println!("Starting GUI load test...");
    
    // 切换到项目根目录
    std::env::set_current_dir("..")?;
    println!("Working directory: {}", std::env::current_dir()?.display());
    
    // 加载配置
    let config_path = Path::new("config.toml");
    println!("Loading config from: {}", config_path.display());
    
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
    
    // 测试搜索所有文件
    println!("\nTesting file search...");
    match tagbox_core::search_files_advanced("*", None, &config).await {
        Ok(result) => {
            println!("✅ Successfully loaded {} files", result.entries.len());
            for (i, entry) in result.entries.iter().enumerate() {
                println!("  {}: {} - {}", i+1, entry.id, entry.title);
            }
        },
        Err(e) => {
            println!("❌ Failed to load files: {}", e);
            println!("Error details: {:?}", e);
            
            // 尝试检查数据库文件是否存在
            if config.database.path.exists() {
                println!("Database file exists at: {}", config.database.path.display());
                let metadata = std::fs::metadata(&config.database.path)?;
                println!("Database size: {} bytes", metadata.len());
            } else {
                println!("Database file does not exist at: {}", config.database.path.display());
            }
        }
    }
    
    Ok(())
}