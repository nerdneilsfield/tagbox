use std::fs;
use tempfile::TempDir;
use tokio;

#[tokio::test]
async fn test_serial_import_works() {
    use tagbox_core::{extract_and_import_file, init_database};

    // 创建临时目录
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage_path = temp_dir.path().join("storage");

    // 创建必要的目录
    fs::create_dir_all(&storage_path).unwrap();

    // 创建配置
    let config_content = format!(
        r#"
[database]
path = "{}"
journal_mode = "WAL"
max_connections = 10
busy_timeout = 5000
sync_mode = "NORMAL"

[storage]
library_path = "{}"
backup_enabled = true
backup_path = "{}/backup"

[import.paths]
storage_dir = "{}"
rename_template = "{{title}}_{{authors}}_{{year}}"
classify_template = "{{category1}}/{{filename}}"

[import.metadata]
prefer_json = true
fallback_pdf = true
default_category = "misc"

[import]
auto_rename = true
naming_template = "{{year}}/{{category}}/{{title}}"
copy_mode = "copy"

[search]
default_limit = 10
enable_fts = true
fts_language = "simple"
fuzzy_search_enabled = true

[hash]
algorithm = "blake2b"
"#,
        db_path.display(),
        storage_path.display(),
        storage_path.display(),
        storage_path.display()
    );

    let config_path = temp_dir.path().join("config.toml");
    fs::write(&config_path, config_content).unwrap();

    // 加载配置
    let config = tagbox_core::load_config(&config_path).await.unwrap();

    // 创建数据库文件
    fs::File::create(&config.database.path).unwrap();

    // 初始化数据库
    init_database(&config.database.path).await.unwrap();

    // 创建测试文件
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "Test content for import").unwrap();

    // 测试串行导入
    println!("Starting serial import test...");
    let result = extract_and_import_file(&file_path, &config).await.unwrap();

    println!("Successfully imported file: {}", result.title);
    assert!(result.path.exists());
}
