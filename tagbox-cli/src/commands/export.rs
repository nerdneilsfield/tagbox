use crate::output::json;
use crate::utils::error::Result;
use std::path::Path;
use tagbox_core::config::AppConfig;
use tagbox_core::types::{FileEntry, SearchOptions};

/// Handle export command
pub async fn handle_export(
    json_output: bool,
    output_file: Option<&Path>,
    config: &AppConfig,
) -> Result<()> {
    log::info!("Exporting all files");

    // Get all files (with a large limit)
    let search_options = Some(SearchOptions {
        offset: 0,
        limit: 1000000, // Very large limit to get all files
        sort_by: Some("created_at".to_string()),
        sort_direction: Some("desc".to_string()),
        include_deleted: false,
    });

    let result = tagbox_core::search_files_advanced("*", search_options, config).await?;

    if json_output {
        let output = if let Some(_path) = output_file {
            json::to_json_string(&result.entries)?
        } else {
            json::print_json(&result.entries)?;
            return Ok(());
        };

        if let Some(path) = output_file {
            std::fs::write(path, output)?;
            println!(
                "Exported {} files to {}",
                result.entries.len(),
                path.display()
            );
        }
    } else {
        // CSV format
        let csv_content = generate_csv(&result.entries)?;

        if let Some(path) = output_file {
            std::fs::write(path, csv_content)?;
            println!(
                "Exported {} files to {}",
                result.entries.len(),
                path.display()
            );
        } else {
            print!("{}", csv_content);
        }
    }

    Ok(())
}

/// Generate CSV content from file entries
fn generate_csv(entries: &[FileEntry]) -> Result<String> {
    let mut csv = String::new();

    // CSV header
    csv.push_str("id,title,authors,year,publisher,source,category1,category2,category3,tags,path,original_filename,hash,created_at,updated_at,is_deleted\n");

    // CSV rows
    for entry in entries {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            escape_csv_field(&entry.id),
            escape_csv_field(&entry.title),
            escape_csv_field(&entry.authors.join("; ")),
            entry.year.map_or_else(|| String::new(), |y| y.to_string()),
            escape_csv_field(&entry.publisher.clone().unwrap_or_default()),
            escape_csv_field(&entry.source.clone().unwrap_or_default()),
            escape_csv_field(&entry.category1),
            escape_csv_field(&entry.category2.clone().unwrap_or_default()),
            escape_csv_field(&entry.category3.clone().unwrap_or_default()),
            escape_csv_field(&entry.tags.join("; ")),
            escape_csv_field(&entry.path.to_string_lossy()),
            escape_csv_field(&entry.original_filename),
            escape_csv_field(&entry.hash),
            entry.created_at.format("%Y-%m-%d %H:%M:%S"),
            entry.updated_at.format("%Y-%m-%d %H:%M:%S"),
            entry.is_deleted
        ));
    }

    Ok(csv)
}

/// Escape CSV field (handle commas, quotes, newlines)
fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}
