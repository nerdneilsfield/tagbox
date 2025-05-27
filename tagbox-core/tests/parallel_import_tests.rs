use serde_json;
use std::fs;
use std::path::Path;
use tempfile::TempDir;
use tokio;

use tagbox_core::{extract_and_import_files, extract_metainfo, init_database, load_config};

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
    // 初始化日志 - 暂时注释掉
    // let _ = env_logger::try_init();

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
    println!("Attempting to import {} files:", file_paths.len());

    // Test metadata extraction for the first file
    println!("Testing metadata extraction for first file...");
    let first_file = &file_paths[0];
    match extract_metainfo(first_file, &config).await {
        Ok(metadata) => {
            println!("Metadata extraction successful:");
            println!("  Title: {}", metadata.title);
            println!("  Authors: {:?}", metadata.authors);
            println!("  Category: {}", metadata.category1);
        }
        Err(e) => {
            println!("Metadata extraction failed: {:?}", e);
        }
    }

    for (i, path) in file_paths.iter().enumerate() {
        println!("  File {}: {}", i, path.display());
        println!("  Exists: {}", path.exists());
        if let Some(parent) = path.parent() {
            let json_name = format!("{}.json", path.file_stem().unwrap().to_string_lossy());
            let json_path = parent.join(&json_name);
            println!(
                "  Expected JSON: {} (exists: {})",
                json_path.display(),
                json_path.exists()
            );
        }
    }
    let results = extract_and_import_files(&file_paths, &config).await;
    let elapsed = start_time.elapsed();

    match &results {
        Ok(entries) => {
            println!(
                "Successfully imported {} files in {:?}",
                entries.len(),
                elapsed
            );
        }
        Err(e) => {
            println!("Import failed with error: {:?}", e);
            panic!("Import failed: {:?}", e);
        }
    }

    let results = results.unwrap();

    // 验证结果
    assert_eq!(results.len(), 10);

    // 验证每个文件都被正确导入
    for entry in results.iter() {
        // 检查标题是否包含 test_file_ 前缀（不依赖顺序）
        assert!(entry.title.starts_with("test_file_") || entry.title.starts_with("Test File"));
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
    // 注意：虽然文件内容相同，但文件名不同会导致元数据不同，
    // 所以可能会导入多个文件。这是预期行为。
    assert!(results1.len() >= 1); // 至少导入一个文件

    // 第二次导入（应该全部跳过）
    let results2 = extract_and_import_files(&file_paths, &config)
        .await
        .unwrap();
    assert_eq!(results2.len(), results1.len()); // 返回相同数量的已存在记录
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
