use std::collections::HashMap;
use std::path::Path;
use tagbox_core::config::AppConfig;
use tagbox_core::types::{FileUpdateRequest, ImportMetadata};
use tempfile::tempdir;

#[tokio::test]
#[ignore]
async fn test_full_flow() {
    // Setup temp environment
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let storage_dir = dir.path().join("files");

    let mut config = AppConfig::default();
    config.database.path = db_path.clone();
    config.import.paths.storage_dir = storage_dir;
    println!("db path: {}", config.database.path.display());
    std::fs::File::create(&config.database.path).unwrap();

    tagbox_core::init_database(&config.database.path)
        .await
        .expect("init db");
    println!("initialized db");

    // Extract metadata
    let meta = tagbox_core::extract_metainfo(Path::new("../test/data/1706.03762v7.pdf"), &config)
        .await
        .expect("extract metadata");
    assert!(!meta.title.is_empty());

    // Import file
    let dummy = ImportMetadata {
        title: String::new(),
        authors: Vec::new(),
        year: None,
        publisher: None,
        source: None,
        category1: "misc".to_string(),
        category2: None,
        category3: None,
        tags: Vec::new(),
        summary: None,
        additional_info: HashMap::new(),
        file_metadata: None,
        type_metadata: None,
    };

    let entry =
        tagbox_core::import_file(Path::new("../test/data/1706.03762v7.pdf"), dummy, &config)
            .await
            .expect("import file");
    println!("imported {}", entry.id);
    assert!(entry.path.exists());

    // Rebuild search index
    tagbox_core::rebuild_search_index(&config)
        .await
        .expect("rebuild index");

    // Search for the file
    let results = tagbox_core::search_files("1706", &config)
        .await
        .expect("search");
    assert!(!results.is_empty());

    // Edit file
    let update = FileUpdateRequest {
        title: Some("Updated Title".to_string()),
        authors: None,
        year: None,
        publisher: None,
        source: None,
        category1: None,
        category2: None,
        category3: None,
        tags: Some(vec!["testtag".to_string()]),
        summary: Some("edited".to_string()),
        is_deleted: None,
        file_metadata: None,
        type_metadata: None,
    };

    tagbox_core::edit_file(&entry.id, update, &config)
        .await
        .expect("edit file");

    // Search again with new title
    let results = tagbox_core::search_files("Updated Title", &config)
        .await
        .expect("search edited");
    assert!(!results.is_empty());
}
