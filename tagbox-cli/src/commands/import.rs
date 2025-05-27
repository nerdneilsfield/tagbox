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
    category_id: Option<String>,
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
    log::info!("Starting import of: {}", path.display());

    if !path.exists() {
        return Err(CliError::FileNotFound(path.to_string_lossy().to_string()));
    }

    let entries = if path.is_file() {
        import_single_file(
            path,
            delete,
            category,
            category_id,
            title,
            authors,
            year,
            publisher,
            source,
            tags,
            summary,
            meta_file,
            config,
        )
        .await?
    } else {
        import_directory(
            path,
            delete,
            category,
            category_id,
            title,
            authors,
            year,
            publisher,
            source,
            tags,
            summary,
            meta_file,
            config,
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
    category_id: Option<String>,
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
        &temp_path,
        delete,
        category,
        category_id,
        title,
        authors,
        year,
        publisher,
        source,
        tags,
        summary,
        meta_file,
        config,
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
    category_id: Option<String>,
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
        category_id,
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
        log::info!("Deleted original file: {}", path.display());
    }

    spinner.finish_with_message("Import completed");
    Ok(vec![entry])
}

/// Import a directory of files
async fn import_directory(
    path: &Path,
    delete: bool,
    _category: Option<String>,
    _category_id: Option<String>,
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

    let progress = create_progress_bar(files.len() as u64, "Importing files");

    // Convert to Path references for the core function
    let file_refs: Vec<&Path> = files.iter().map(|p| p.as_path()).collect();

    // Use the core batch import function
    let entries = tagbox_core::extract_and_import_files(&file_refs, config).await?;

    // Handle delete option for successfully imported files
    if delete && !entries.is_empty() {
        progress.set_message("Deleting original files...");
        for file in &files {
            if let Err(e) = std::fs::remove_file(file) {
                log::warn!("Failed to delete {}: {}", file.display(), e);
            }
        }

        // Remove empty directories if all files were deleted
        if let Err(e) = std::fs::remove_dir_all(path) {
            log::warn!("Failed to remove directory {}: {}", path.display(), e);
        }
    }

    progress.finish_with_message("Import completed");
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
    _category_id: Option<String>, // TODO: implement category lookup by ID
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

    if let Some(category) = category {
        metadata.category1 = category;
    }

    if let Some(tags) = tags {
        metadata.tags = tags.split(',').map(|s| s.trim().to_string()).collect();
    }

    if let Some(summary) = summary {
        metadata.summary = Some(summary);
    }

    Ok(())
}
