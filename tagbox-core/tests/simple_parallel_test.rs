use std::fs;
use std::path::Path;
use tempfile::TempDir;
use tokio;

#[tokio::test]
async fn test_simple_parallel_import() {
    use tagbox_core::{extract_and_import_files, init_database};

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
    let files_dir = temp_dir.path().join("test_files");
    fs::create_dir_all(&files_dir).unwrap();

    let mut test_files = Vec::new();
    for i in 0..3 {
        let file_path = files_dir.join(format!("test_{}.txt", i));
        fs::write(&file_path, format!("Test content {}", i)).unwrap();
        test_files.push(file_path);
    }

    let file_paths: Vec<&Path> = test_files.iter().map(|p| p.as_path()).collect();

    // 测试并行导入
    println!("Starting parallel import test...");
    let results = extract_and_import_files(&file_paths, &config)
        .await
        .unwrap();

    // 验证结果
    assert_eq!(results.len(), 3);
    println!("Successfully imported {} files", results.len());
}
