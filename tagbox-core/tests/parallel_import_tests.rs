use serde_json;
use std::fs;
use std::path::Path;
use tempfile::TempDir;
use tokio;

use tagbox_core::{extract_and_import_files, init_database, load_config};

/// 创建测试配置文件
fn create_test_config(temp_dir: &Path) -> std::path::PathBuf {
    let config_path = temp_dir.join("config.toml");
    let db_path = temp_dir.join("test.db");
    let storage_path = temp_dir.join("storage");

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

    // 创建必要的目录
    fs::create_dir_all(db_path.parent().unwrap()).unwrap();
    fs::create_dir_all(&storage_path).unwrap();

    fs::write(&config_path, config_content).unwrap();
    config_path
}

/// 创建测试文件
fn create_test_files(temp_dir: &Path, count: usize) -> Vec<std::path::PathBuf> {
    // 使用配置中的 storage 目录
    let files_dir = temp_dir.join("storage").join("test_files");
    fs::create_dir_all(&files_dir).unwrap();

    let mut files = Vec::new();

    for i in 0..count {
        // 创建PDF文件（使用简单的PDF header来模拟）
        let file_path = files_dir.join(format!("test_file_{}.pdf", i));
        let pdf_content = format!(
            "%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n3 0 obj\n<< /Type /Page /Parent 2 0 R /Resources << >> /MediaBox [0 0 612 792] >>\nendobj\nxref\n0 4\n0000000000 65535 f\n0000000009 00000 n\n0000000058 00000 n\n0000000115 00000 n\ntrailer\n<< /Size 4 /Root 1 0 R >>\nstartxref\n223\n%%EOF\nTest content for file {}", 
            i
        );
        fs::write(&file_path, pdf_content).unwrap();

        // 创建对应的元数据JSON文件
        let json_path = files_dir.join(format!("test_file_{}.json", i));
        let metadata = serde_json::json!({
            "title": format!("Test File {}", i),
            "authors": [format!("Author {}", i)],
            "year": 2024 + (i as i32 % 2),
            "publisher": format!("Publisher {}", i % 3),
            "category1": if i % 2 == 0 { "技术" } else { "文学" },
            "tags": [format!("tag{}", i), "test"],
            "summary": format!("This is a test file number {} for parallel import testing", i)
        });
        fs::write(&json_path, serde_json::to_string_pretty(&metadata).unwrap()).unwrap();

        files.push(file_path);
    }

    files
}

#[tokio::test]
async fn test_parallel_import_multiple_files() {
    // 初始化测试环境
    let temp_dir = TempDir::new().unwrap();
    let config_path = create_test_config(temp_dir.path());
    let config = load_config(&config_path).await.unwrap();

    // 确保数据库文件存在
    if let Some(parent) = config.database.path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::File::create(&config.database.path).unwrap();

    // 初始化数据库
    init_database(&config.database.path).await.unwrap();

    // 创建测试文件
    let test_files = create_test_files(temp_dir.path(), 10);
    let file_paths: Vec<&Path> = test_files.iter().map(|p| p.as_path()).collect();

    // 测试并行导入
    let start_time = std::time::Instant::now();
    let results = extract_and_import_files(&file_paths, &config)
        .await
        .unwrap();
    let elapsed = start_time.elapsed();

    println!("Imported {} files in {:?}", results.len(), elapsed);

    // 验证结果
    assert_eq!(results.len(), 10);

    // 验证每个文件都被正确导入
    for (i, entry) in results.iter().enumerate() {
        assert!(entry.title.contains(&format!("test_file_{}", i)));
        assert!(entry.path.exists());
    }
}

#[tokio::test]
async fn test_parallel_import_with_failures() {
    // 初始化测试环境
    let temp_dir = TempDir::new().unwrap();
    let config_path = create_test_config(temp_dir.path());
    let config = load_config(&config_path).await.unwrap();

    // 确保数据库文件存在
    if let Some(parent) = config.database.path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::File::create(&config.database.path).unwrap();

    // 初始化数据库
    init_database(&config.database.path).await.unwrap();

    // 创建测试文件，包括一些不存在的文件
    let mut test_files = create_test_files(temp_dir.path(), 5);

    // 添加一些不存在的文件路径
    test_files.push(temp_dir.path().join("non_existent_1.txt"));
    test_files.push(temp_dir.path().join("non_existent_2.txt"));

    let file_paths: Vec<&Path> = test_files.iter().map(|p| p.as_path()).collect();

    // 测试并行导入（应该部分成功）
    let results = extract_and_import_files(&file_paths, &config)
        .await
        .unwrap();

    // 验证只有存在的文件被导入
    assert_eq!(results.len(), 5);
}

#[tokio::test]
async fn test_parallel_import_duplicate_files() {
    // 初始化测试环境
    let temp_dir = TempDir::new().unwrap();
    let config_path = create_test_config(temp_dir.path());
    let config = load_config(&config_path).await.unwrap();

    // 确保数据库文件存在
    if let Some(parent) = config.database.path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::File::create(&config.database.path).unwrap();

    // 初始化数据库
    init_database(&config.database.path).await.unwrap();

    // 创建相同内容的文件（将被去重）
    let files_dir = temp_dir.path().join("test_files");
    fs::create_dir_all(&files_dir).unwrap();

    let content = "Duplicate content for testing";
    let mut files = Vec::new();

    for i in 0..5 {
        let file_path = files_dir.join(format!("duplicate_{}.txt", i));
        fs::write(&file_path, content).unwrap();
        files.push(file_path);
    }

    let file_paths: Vec<&Path> = files.iter().map(|p| p.as_path()).collect();

    // 第一次导入
    let results1 = extract_and_import_files(&file_paths, &config)
        .await
        .unwrap();
    assert_eq!(results1.len(), 1); // 只有第一个文件被导入，其他的因为哈希相同被跳过

    // 第二次导入（应该全部跳过）
    let results2 = extract_and_import_files(&file_paths, &config)
        .await
        .unwrap();
    assert_eq!(results2.len(), 1); // 返回已存在的记录
}

#[tokio::test]
async fn test_parallel_import_performance() {
    // 初始化测试环境
    let temp_dir = TempDir::new().unwrap();
    let config_path = create_test_config(temp_dir.path());
    let config = load_config(&config_path).await.unwrap();

    // 确保数据库文件存在
    if let Some(parent) = config.database.path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::File::create(&config.database.path).unwrap();

    // 初始化数据库
    init_database(&config.database.path).await.unwrap();

    // 创建较多的测试文件来测试性能
    let test_files = create_test_files(temp_dir.path(), 50);
    let file_paths: Vec<&Path> = test_files.iter().map(|p| p.as_path()).collect();

    // 测试并行导入性能
    let start_time = std::time::Instant::now();
    let results = extract_and_import_files(&file_paths, &config)
        .await
        .unwrap();
    let parallel_time = start_time.elapsed();

    println!(
        "Parallel import: {} files in {:?} ({:.2} files/sec)",
        results.len(),
        parallel_time,
        results.len() as f64 / parallel_time.as_secs_f64()
    );

    assert_eq!(results.len(), 50);
}
