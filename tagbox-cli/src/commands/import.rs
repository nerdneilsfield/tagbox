use crate::output::progress::{create_progress_bar, create_spinner};
use crate::utils::error::{CliError, Result};
use std::path::{Path, PathBuf};
use tagbox_core::config::AppConfig;
use tagbox_core::types::{FileEntry, ImportMetadata};
use url::Url;

/// Handle file import command
pub async fn handle_import(
    path: &Path,
    delete: bool,
    category: Option<String>,
    title: Option<String>,
    authors: Option<String>,
    year: Option<i32>,
    publisher: Option<String>,
    source: Option<String>,
    tags: Option<String>,
    summary: Option<String>,
    meta_file: Option<PathBuf>,
    interactive: bool,
    config: &AppConfig,
) -> Result<()> {
    log::info!("Starting import of: {}", path.display());

    if !path.exists() {
        return Err(CliError::FileNotFound(path.to_string_lossy().to_string()));
    }

    let entries = if path.is_file() {
        import_single_file(
            path, delete, category, title, authors, year, publisher, source, tags, summary,
            meta_file, config,
        )
        .await?
    } else {
        import_directory(
            path, delete, category, title, authors, year, publisher, source, tags, summary,
            meta_file, config,
        )
        .await?
    };

    // Return the entries for stdio mode
    if entries.is_empty() {
        return Err(CliError::CommandFailed("No files imported".to_string()));
    }

    println!("Successfully imported {} file(s)", entries.len());
    for entry in &entries {
        println!("  {} -> {}", entry.original_filename, entry.id);
    }

    Ok(())
}

/// Handle URL import command
pub async fn handle_import_url(
    url: &str,
    rename: Option<String>,
    delete: bool,
    category: Option<String>,
    title: Option<String>,
    authors: Option<String>,
    year: Option<i32>,
    publisher: Option<String>,
    source: Option<String>,
    tags: Option<String>,
    summary: Option<String>,
    meta_file: Option<PathBuf>,
    config: &AppConfig,
) -> Result<()> {
    log::info!("Downloading from URL: {}", url);

    // Download file
    let temp_path = download_file(url, rename).await?;

    // Import the downloaded file
    let result = import_single_file(
        &temp_path, delete, category, title, authors, year, publisher, source, tags, summary,
        meta_file, config,
    )
    .await;

    // Clean up temp file
    if let Err(e) = std::fs::remove_file(&temp_path) {
        log::warn!(
            "Failed to clean up temp file {}: {}",
            temp_path.display(),
            e
        );
    }

    match result {
        Ok(entries) => {
            println!("Successfully imported from URL: {}", entries[0].id);
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Import a single file
async fn import_single_file(
    path: &Path,
    delete: bool,
    category: Option<String>,
    title: Option<String>,
    authors: Option<String>,
    year: Option<i32>,
    publisher: Option<String>,
    source: Option<String>,
    tags: Option<String>,
    summary: Option<String>,
    meta_file: Option<PathBuf>,
    config: &AppConfig,
) -> Result<Vec<FileEntry>> {
    let spinner = create_spinner("Extracting metadata...");

    // Load metadata from file if provided
    let mut metadata = if let Some(meta_path) = meta_file {
        load_metadata_from_file(&meta_path).await?
    } else {
        tagbox_core::extract_metainfo(path, config).await?
    };

    // Override with command line arguments
    apply_metadata_overrides(
        &mut metadata,
        category,
        title,
        authors,
        year,
        publisher,
        source,
        tags,
        summary,
    )?;

    spinner.set_message("Importing file...");
    let entry = tagbox_core::import_file(path, metadata, config).await?;

    // Handle delete option
    if delete {
        spinner.set_message("Deleting original file...");
        std::fs::remove_file(path)?;
        log::debug!("Deleted original file: {}", path.display());
    }

    spinner.finish_with_message("Import completed");
    Ok(vec![entry])
}

/// Import a directory of files
async fn import_directory(
    path: &Path,
    delete: bool,
    _category: Option<String>,
    _title: Option<String>,
    _authors: Option<String>,
    _year: Option<i32>,
    _publisher: Option<String>,
    _source: Option<String>,
    _tags: Option<String>,
    _summary: Option<String>,
    _meta_file: Option<PathBuf>,
    config: &AppConfig,
) -> Result<Vec<FileEntry>> {
    // Collect all files in directory
    let files = collect_files(path)?;

    if files.is_empty() {
        return Ok(Vec::new());
    }

    println!("Found {} files to import", files.len());

    // 阶段1：并行提取元数据
    let metadata_progress = create_progress_bar(files.len() as u64, "Extracting metadata");

    // 使用基础的 tokio::spawn 来实现并行处理
    let mut metadata_tasks = Vec::new();

    for file in &files {
        let file_clone = file.clone();
        let config_clone = config.clone();
        let progress_clone = metadata_progress.clone();

        let task = tokio::spawn(async move {
            let result = tagbox_core::extract_metainfo(&file_clone, &config_clone).await;
            progress_clone.inc(1);
            (file_clone, result)
        });

        metadata_tasks.push(task);
    }

    // 等待所有元数据提取任务完成
    let mut metadata_pairs = Vec::new();
    let mut extraction_errors = Vec::new();

    for task in metadata_tasks {
        match task.await {
            Ok((file_path, metadata_result)) => match metadata_result {
                Ok(metadata) => {
                    metadata_pairs.push((file_path, metadata));
                }
                Err(e) => {
                    log::warn!(
                        "Failed to extract metadata from {}: {}",
                        file_path.display(),
                        e
                    );
                    extraction_errors.push(e);
                }
            },
            Err(join_err) => {
                log::error!("Task join error: {}", join_err);
                // 将join错误转换为TagboxError
                let tagbox_err = tagbox_core::errors::TagboxError::ImportError(format!(
                    "Task failed: {}",
                    join_err
                ));
                extraction_errors.push(tagbox_err);
            }
        }
    }

    let metadata_finish_msg = format!(
        "Metadata extraction completed: {} succeeded, {} failed",
        metadata_pairs.len(),
        extraction_errors.len()
    );
    metadata_progress.finish_with_message(metadata_finish_msg);

    if metadata_pairs.is_empty() {
        return Err(CliError::CommandFailed(
            "No files had extractable metadata".to_string(),
        ));
    }

    // 阶段2：串行导入到数据库
    let import_progress = create_progress_bar(metadata_pairs.len() as u64, "Importing to database");

    let mut entries = Vec::new();
    let mut import_errors = Vec::new();

    for (file_path, metadata) in metadata_pairs {
        let filename = file_path.file_name().unwrap_or_default().to_string_lossy();
        let import_msg = format!("Importing {}", filename);
        import_progress.set_message(import_msg);

        match tagbox_core::import_file(&file_path, metadata, config).await {
            Ok(entry) => {
                entries.push(entry);
                import_progress.inc(1);
            }
            Err(e) => {
                log::warn!("Failed to import {}: {}", file_path.display(), e);
                import_errors.push((file_path, e));
                import_progress.inc(1);
            }
        }
    }

    let import_finish_msg = format!(
        "Import completed: {} succeeded, {} failed",
        entries.len(),
        import_errors.len()
    );
    import_progress.finish_with_message(import_finish_msg);

    // Handle delete option for successfully imported files
    if delete && !entries.is_empty() {
        let delete_progress = create_progress_bar(entries.len() as u64, "Deleting original files");

        for entry in &entries {
            // 找到对应的原始文件路径来删除
            if let Some(original_file) = files.iter().find(|f| {
                f.file_name().unwrap_or_default().to_string_lossy() == entry.original_filename
            }) {
                if let Err(e) = std::fs::remove_file(original_file) {
                    log::warn!("Failed to delete {}: {}", original_file.display(), e);
                }
            }
            delete_progress.inc(1);
        }

        delete_progress.finish_with_message("File deletion completed");

        // Remove empty directories if all files were deleted
        if entries.len() == files.len() {
            if let Err(e) = std::fs::remove_dir_all(path) {
                log::warn!("Failed to remove directory {}: {}", path.display(), e);
            }
        }
    }

    // 显示最终统计
    println!("Import summary:");
    println!("  Total files found: {}", files.len());
    println!(
        "  Metadata extraction failures: {}",
        extraction_errors.len()
    );
    println!("  Import failures: {}", import_errors.len());
    println!("  Successfully imported: {}", entries.len());

    Ok(entries)
}

/// Collect all files in a directory recursively
fn collect_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_files_recursive(dir, &mut files)?;
    Ok(files)
}

/// Recursively collect files from directory
fn collect_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            files.push(path);
        } else if path.is_dir() {
            collect_files_recursive(&path, files)?;
        }
    }
    Ok(())
}

/// Download file from URL
async fn download_file(url: &str, rename: Option<String>) -> Result<PathBuf> {
    let parsed_url = Url::parse(url)?;

    let filename = match rename {
        Some(name) => name,
        None => {
            // Extract filename from URL
            parsed_url
                .path_segments()
                .and_then(|segments| segments.last())
                .unwrap_or("download")
                .to_string()
        }
    };

    // Create temp directory
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(filename);

    // Download file
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;

    // Write to temp file
    std::fs::write(&temp_path, bytes)?;

    Ok(temp_path)
}

/// Load metadata from JSON file
async fn load_metadata_from_file(path: &Path) -> Result<ImportMetadata> {
    let content = std::fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(CliError::Json)
}

/// Apply command line metadata overrides
fn apply_metadata_overrides(
    metadata: &mut ImportMetadata,
    category: Option<String>,
    title: Option<String>,
    authors: Option<String>,
    year: Option<i32>,
    publisher: Option<String>,
    source: Option<String>,
    tags: Option<String>,
    summary: Option<String>,
) -> Result<()> {
    if let Some(title) = title {
        metadata.title = title;
    }

    if let Some(authors) = authors {
        metadata.authors = authors.split(',').map(|s| s.trim().to_string()).collect();
    }

    if let Some(year) = year {
        metadata.year = Some(year);
    }

    if let Some(publisher) = publisher {
        metadata.publisher = Some(publisher);
    }

    if let Some(source) = source {
        metadata.source = Some(source);
    }

    // Handle category assignment using new path-based format
    if let Some(category_path) = category {
        let (cat1, cat2, cat3) = tagbox_core::utils::parse_category_string(&category_path)
            .map_err(|e| CliError::CommandFailed(format!("Invalid category format: {}", e)))?;

        metadata.category1 = cat1;
        metadata.category2 = cat2;
        metadata.category3 = cat3;
    }

    if let Some(tags) = tags {
        metadata.tags = tags.split(',').map(|s| s.trim().to_string()).collect();
    }

    if let Some(summary) = summary {
        metadata.summary = Some(summary);
    }

    Ok(())
}
