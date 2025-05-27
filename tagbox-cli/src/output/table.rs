use crate::utils::error::Result;
use tabled::{builder::Builder, settings::Style};
use tagbox_core::types::FileEntry;

/// Print file entries as a formatted table
pub fn print_file_table(entries: &[FileEntry], columns: Option<&str>) -> Result<()> {
    if entries.is_empty() {
        println!("No files found.");
        return Ok(());
    }

    let columns = parse_columns(columns);
    let mut builder = Builder::default();

    // Add header
    builder.push_record(columns.iter().map(|c| column_header(c)));

    // Add data rows
    for entry in entries {
        builder.push_record(columns.iter().map(|c| format_column_value(entry, c)));
    }

    let table = builder.build().with(Style::rounded()).to_string();
    println!("{}", table);

    Ok(())
}

/// Parse column specification
fn parse_columns(columns: Option<&str>) -> Vec<String> {
    match columns {
        Some(cols) => cols.split(',').map(|s| s.trim().to_string()).collect(),
        None => vec![
            "id".to_string(),
            "title".to_string(),
            "authors".to_string(),
            "path".to_string(),
        ],
    }
}

/// Get column header name
fn column_header(column: &str) -> String {
    match column {
        "id" => "ID".to_string(),
        "title" => "Title".to_string(),
        "authors" => "Authors".to_string(),
        "year" => "Year".to_string(),
        "publisher" => "Publisher".to_string(),
        "source" => "Source".to_string(),
        "path" => "Path".to_string(),
        "category1" => "Category".to_string(),
        "tags" => "Tags".to_string(),
        "created_at" => "Created".to_string(),
        "updated_at" => "Updated".to_string(),
        _ => column.to_string(),
    }
}

/// Format column value for display
fn format_column_value(entry: &FileEntry, column: &str) -> String {
    match column {
        "id" => entry.id.clone(),
        "title" => entry.title.clone(),
        "authors" => entry.authors.join(", "),
        "year" => entry
            .year
            .map_or_else(|| "-".to_string(), |y| y.to_string()),
        "publisher" => entry.publisher.clone().unwrap_or_else(|| "-".to_string()),
        "source" => entry.source.clone().unwrap_or_else(|| "-".to_string()),
        "path" => entry.path.to_string_lossy().to_string(),
        "category1" => entry.category1.clone(),
        "tags" => entry.tags.join(", "),
        "created_at" => entry.created_at.format("%Y-%m-%d %H:%M").to_string(),
        "updated_at" => entry.updated_at.format("%Y-%m-%d %H:%M").to_string(),
        _ => "-".to_string(),
    }
}
