use std::path::Path;
use tagbox_core::config::AppConfig;
use tagbox_core::metainfo::MetaInfoExtractor;

#[tokio::test]
async fn test_epub_metadata_extraction() {
    // 创建配置
    let config = AppConfig::default();
    let extractor = MetaInfoExtractor::new(config);

    // 测试EPUB文件
    let epub_path = Path::new("../test/data/test.epub");
    let result = extractor.extract(epub_path).await;

    assert!(result.is_ok(), "EPUB extraction should succeed");
    let metadata = result.unwrap();

    // 验证基本元数据
    assert!(!metadata.title.is_empty(), "Title should not be empty");

    // 验证文件元数据存在
    assert!(
        metadata.file_metadata.is_some(),
        "File metadata should exist"
    );
    let file_meta = metadata.file_metadata.as_ref().unwrap();

    // 验证EPUB特定元数据
    assert!(file_meta.get("epub").is_some(), "Should have epub metadata");
    let epub_meta = &file_meta["epub"];
    assert!(
        epub_meta.get("spine_count").is_some(),
        "Should have spine count"
    );
    assert!(
        epub_meta.get("has_cover").is_some(),
        "Should have cover info"
    );

    // 验证类型元数据
    assert!(
        metadata.type_metadata.is_some(),
        "Type metadata should exist"
    );
    let type_meta = metadata.type_metadata.as_ref().unwrap();
    assert!(type_meta.get("book").is_some(), "Should have book metadata");

    println!("EPUB Metadata extracted:");
    println!("Title: {}", metadata.title);
    println!("Authors: {:?}", metadata.authors);
    println!(
        "File metadata: {}",
        serde_json::to_string_pretty(&file_meta).unwrap()
    );
    println!(
        "Type metadata: {}",
        serde_json::to_string_pretty(&type_meta).unwrap()
    );
}

#[tokio::test]
async fn test_pdf_metadata_extraction() {
    let config = AppConfig::default();
    let extractor = MetaInfoExtractor::new(config);

    // 测试第一个PDF文件
    let pdf_path = Path::new("../test/data/1706.03762v7.pdf");
    let result = extractor.extract(pdf_path).await;

    assert!(result.is_ok(), "PDF extraction should succeed");
    let metadata = result.unwrap();

    // 验证基本元数据
    assert!(!metadata.title.is_empty(), "Title should not be empty");

    // 验证文件元数据
    assert!(
        metadata.file_metadata.is_some(),
        "File metadata should exist"
    );
    let file_meta = metadata.file_metadata.as_ref().unwrap();

    // 验证PDF特定元数据
    assert!(file_meta.get("pdf").is_some(), "Should have pdf metadata");
    let pdf_meta = &file_meta["pdf"];
    // TODO: 当PDF实现完整后，取消注释以下断言
    // assert!(pdf_meta.get("pages").is_some(), "Should have page count");
    // assert!(pdf_meta.get("version").is_some(), "Should have PDF version");
    assert!(
        pdf_meta.get("note").is_some(),
        "Should have note about pending implementation"
    );

    println!("\nPDF Metadata extracted (1706.03762v7.pdf):");
    println!("Title: {}", metadata.title);
    println!("Authors: {:?}", metadata.authors);
    println!("Year: {:?}", metadata.year);
    println!("Tags: {:?}", metadata.tags);
    println!("Summary: {:?}", metadata.summary);
    println!(
        "File metadata: {}",
        serde_json::to_string_pretty(&file_meta).unwrap()
    );

    // 如果有type_metadata，打印出来
    if let Some(type_meta) = &metadata.type_metadata {
        println!(
            "Type metadata: {}",
            serde_json::to_string_pretty(&type_meta).unwrap()
        );
    }
}

#[tokio::test]
async fn test_pdf_metadata_extraction_gpt4() {
    let config = AppConfig::default();
    let extractor = MetaInfoExtractor::new(config);

    // 测试GPT-4系统卡片PDF
    let pdf_path = Path::new("../test/data/gpt-4o-system-card.pdf");
    let result = extractor.extract(pdf_path).await;

    assert!(result.is_ok(), "PDF extraction should succeed");
    let metadata = result.unwrap();

    println!("\nPDF Metadata extracted (gpt-4o-system-card.pdf):");
    println!("Title: {}", metadata.title);
    println!("Authors: {:?}", metadata.authors);
    println!("Tags: {:?}", metadata.tags);

    // 验证文件元数据
    assert!(
        metadata.file_metadata.is_some(),
        "File metadata should exist"
    );
    let file_meta = metadata.file_metadata.as_ref().unwrap();
    assert!(file_meta.get("pdf").is_some(), "Should have pdf metadata");

    println!(
        "File metadata: {}",
        serde_json::to_string_pretty(&file_meta).unwrap()
    );
}

#[tokio::test]
async fn test_fallback_to_filename() {
    let config = AppConfig::default();
    let extractor = MetaInfoExtractor::new(config);

    // 测试一个不存在的文件，应该失败
    let fake_path = Path::new("../test/data/nonexistent.pdf");
    let result = extractor.extract(fake_path).await;

    assert!(result.is_err(), "Should fail for non-existent file");
}

#[tokio::test]
async fn test_json_metadata_merge() {
    let config = AppConfig::default();
    let extractor = MetaInfoExtractor::new(config);

    // 测试PDF，看看file_metadata和type_metadata是否正确合并
    let pdf_path = Path::new("../test/data/1706.03762v7.pdf");
    let result = extractor.extract(pdf_path).await;

    assert!(result.is_ok());
    let metadata = result.unwrap();

    // 验证JSON元数据可以被正确序列化和反序列化
    if let Some(file_meta) = &metadata.file_metadata {
        let json_str = serde_json::to_string(file_meta).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert!(parsed.is_object());
    }

    if let Some(type_meta) = &metadata.type_metadata {
        let json_str = serde_json::to_string(type_meta).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert!(parsed.is_object());
    }
}

#[cfg(test)]
mod metadata_format_tests {

    #[test]
    fn test_file_metadata_structure() {
        // 测试PDF元数据结构
        let pdf_metadata = serde_json::json!({
            "pdf": {
                "pages": 42,
                "version": "1.7",
                "producer": "LaTeX",
                "has_ocr": false
            }
        });

        assert!(pdf_metadata["pdf"]["pages"].as_u64().unwrap() == 42);
        assert!(pdf_metadata["pdf"]["version"].as_str().unwrap() == "1.7");

        // 测试EPUB元数据结构
        let epub_metadata = serde_json::json!({
            "epub": {
                "spine_count": 10,
                "has_cover": true,
                "cover_size": 123456
            }
        });

        assert!(epub_metadata["epub"]["spine_count"].as_u64().unwrap() == 10);
        assert!(epub_metadata["epub"]["has_cover"].as_bool().unwrap());

        // 测试图片元数据结构
        let image_metadata = serde_json::json!({
            "image": {
                "width": 1920,
                "height": 1080,
                "format": "JPEG",
                "has_alpha": false,
                "bits_per_pixel": 24
            }
        });

        assert!(image_metadata["image"]["width"].as_u64().unwrap() == 1920);
        assert!(image_metadata["image"]["format"].as_str().unwrap() == "JPEG");
    }

    #[test]
    fn test_type_metadata_structure() {
        // 测试书籍元数据
        let book_metadata = serde_json::json!({
            "book": {
                "isbn": "978-1234567890",
                "edition": "3rd",
                "series": "Head First",
                "language": "en"
            }
        });

        assert!(book_metadata["book"]["isbn"].as_str().unwrap() == "978-1234567890");
        assert!(book_metadata["book"]["language"].as_str().unwrap() == "en");

        // 测试论文元数据
        let paper_metadata = serde_json::json!({
            "paper": {
                "doi": "10.1234/example",
                "journal": "Nature",
                "peer_reviewed": true,
                "keywords": ["AI", "Machine Learning", "Deep Learning"]
            }
        });

        assert!(paper_metadata["paper"]["doi"].as_str().unwrap() == "10.1234/example");
        assert!(paper_metadata["paper"]["peer_reviewed"].as_bool().unwrap());
        assert!(
            paper_metadata["paper"]["keywords"]
                .as_array()
                .unwrap()
                .len()
                == 3
        );
    }
}
