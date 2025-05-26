use std::collections::HashMap;
use tagbox_core::{config::AppConfig, pathgen::PathGenerator, types::ImportMetadata};

fn sample_metadata() -> ImportMetadata {
    ImportMetadata {
        title: "Rust Book".to_string(),
        authors: vec!["Steve".to_string(), "Carol".to_string()],
        year: Some(2021),
        publisher: Some("Rustaceans".to_string()),
        source: None,
        category1: "books".to_string(),
        category2: None,
        category3: None,
        tags: vec![],
        summary: None,
        additional_info: HashMap::new(),
        file_metadata: None,
        type_metadata: None,
    }
}

#[test]
fn test_generate_filename_and_path() {
    let config = AppConfig::default();
    let generator = PathGenerator::new(config.clone());
    let meta = sample_metadata();
    let filename = generator.generate_filename("orig.pdf", &meta).unwrap();
    assert!(filename.ends_with(".pdf"));
    assert!(filename.contains("Rust Book"));

    let path = generator.generate_path(&filename, &meta).unwrap();
    assert!(path.starts_with(config.import.paths.storage_dir));
    assert!(path.to_string_lossy().contains(&filename));
}
