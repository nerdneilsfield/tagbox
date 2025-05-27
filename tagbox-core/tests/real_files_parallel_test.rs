use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio;

#[tokio::test]
async fn test_parallel_import_real_files() {
    use tagbox_core::{extract_and_import_files, init_database, load_config};

    // 使用实际的测试文件
    let test_files = vec![
        PathBuf::from("../test/data/1706.03762v7.pdf"),
        PathBuf::from("../test/data/gpt-4o-system-card.pdf"),
    ];

    // 确保测试文件存在
    for file in &test_files {
        if !file.exists() {
            eprintln!("Skipping test: file {} not found", file.display());
            return;
        }
    }

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
    let config = load_config(&config_path).await.unwrap();

    // 创建数据库文件
    fs::File::create(&config.database.path).unwrap();

    // 初始化数据库
    init_database(&config.database.path).await.unwrap();

    // 准备文件路径
    let file_paths: Vec<&std::path::Path> = test_files.iter().map(|p| p.as_path()).collect();

    // 测试并行导入
    println!("Starting parallel import of real files...");
    let start_time = std::time::Instant::now();

    let results = extract_and_import_files(&file_paths, &config)
        .await
        .unwrap();

    let elapsed = start_time.elapsed();
    println!(
        "Imported {} files in {:?} ({:.2} files/sec)",
        results.len(),
        elapsed,
        results.len() as f64 / elapsed.as_secs_f64()
    );

    // 验证结果
    assert_eq!(results.len(), 2);

    // 验证文件被正确导入
    for entry in &results {
        assert!(entry.path.exists());
        println!("Imported: {} -> {}", entry.title, entry.path.display());
    }
}
