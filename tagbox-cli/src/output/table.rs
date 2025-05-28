use crate::utils::error::Result;
use colored::*;
use serde_json::Value;
use tabled::{
    builder::Builder,
    settings::{object::Columns, Modify, Style, Width},
};
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

    let table = builder
        .build()
        .with(Style::rounded())
        // ID column - no wrapping to allow easy copying
        .with(Modify::new(Columns::single(1)).with(Width::wrap(30).keep_words(true))) // Title column
        .with(Modify::new(Columns::single(2)).with(Width::wrap(20).keep_words(true))) // Authors column
        .with(Modify::new(Columns::single(3)).with(Width::wrap(25).keep_words(true))) // Tags column
        .with(Modify::new(Columns::single(4)).with(Width::wrap(15).keep_words(true))) // Category column
        .to_string();
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
            "tags".to_string(),
            "category1".to_string(),
            "created_at".to_string(),
            "updated_at".to_string(),
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

/// Print file entry as a two-column table (property-value pairs)
pub fn print_preview_table(entry: &FileEntry) -> Result<()> {
    let mut builder = Builder::default();

    // Collect all data as owned strings to avoid lifetime issues
    let mut rows: Vec<[String; 2]> = Vec::new();

    // Add header
    rows.push(["Property".to_string(), "Value".to_string()]);

    // Add data rows
    rows.push(["ID".to_string(), entry.id.clone()]);
    rows.push(["Title".to_string(), entry.title.clone()]);

    if !entry.authors.is_empty() {
        rows.push(["Authors".to_string(), entry.authors.join(", ")]);
    }

    if let Some(year) = entry.year {
        rows.push(["Year".to_string(), year.to_string()]);
    }

    if let Some(publisher) = &entry.publisher {
        rows.push(["Publisher".to_string(), publisher.clone()]);
    }

    if let Some(source) = &entry.source {
        rows.push(["Source".to_string(), source.clone()]);
    }

    rows.push(["Category".to_string(), entry.category1.clone()]);

    if let Some(cat2) = &entry.category2 {
        rows.push(["Subcategory".to_string(), cat2.clone()]);
    }

    if let Some(cat3) = &entry.category3 {
        rows.push(["Sub-subcategory".to_string(), cat3.clone()]);
    }

    if !entry.tags.is_empty() {
        rows.push(["Tags".to_string(), entry.tags.join(", ")]);
    }

    if let Some(summary) = &entry.summary {
        rows.push(["Summary".to_string(), summary.clone()]);
    }

    rows.push([
        "Original filename".to_string(),
        entry.original_filename.clone(),
    ]);
    rows.push(["Hash".to_string(), entry.hash.clone()]);

    if let Some(current_hash) = &entry.current_hash {
        if current_hash != &entry.hash {
            rows.push([
                "Current hash".to_string(),
                format!("{} (CHANGED!)", current_hash),
            ]);
        }
    }

    rows.push([
        "Created".to_string(),
        entry.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
    ]);
    rows.push([
        "Updated".to_string(),
        entry.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
    ]);

    if let Some(accessed) = entry.last_accessed {
        rows.push([
            "Last accessed".to_string(),
            accessed.format("%Y-%m-%d %H:%M:%S").to_string(),
        ]);
    }

    if entry.is_deleted {
        rows.push(["Status".to_string(), "DELETED".to_string()]);
    }

    rows.push(["Path".to_string(), entry.path.to_string_lossy().to_string()]);

    // Parse and display file_metadata JSON if available
    if let Some(file_metadata) = &entry.file_metadata {
        rows.push(["".to_string(), "".to_string()]); // Empty row as separator
        rows.push(["File Metadata".to_string(), "".to_string()]);

        // Special handling for PDF text content
        if let Some(pdf_data) = file_metadata.get("pdf") {
            if let Some(text_preview) = pdf_data.get("text_preview") {
                if let Some(preview_str) = text_preview.as_str() {
                    if !preview_str.trim().is_empty() {
                        rows.push(["  Text Preview".to_string(), preview_str.to_string()]);
                    }
                }
            }

            // Show if full text is available
            if let Some(has_text) = pdf_data.get("has_text") {
                if let Some(has_text_bool) = has_text.as_bool() {
                    if has_text_bool {
                        if let Some(text_length) = pdf_data.get("text_length") {
                            if let Some(length) = text_length.as_u64() {
                                rows.push([
                                    "  Full Text Length".to_string(),
                                    format!("{} characters", length),
                                ]);
                            }
                        }
                    }
                }
            }
        }

        add_json_fields_to_rows(&mut rows, file_metadata, "");
    }

    // Parse and display type_metadata JSON if available
    if let Some(type_metadata) = &entry.type_metadata {
        rows.push(["".to_string(), "".to_string()]); // Empty row as separator
        rows.push(["Type Metadata".to_string(), "".to_string()]);

        add_json_fields_to_rows(&mut rows, type_metadata, "");
    }

    // Now add all rows to builder
    for row in &rows {
        builder.push_record(row);
    }

    let table = builder
        .build()
        .with(Style::rounded())
        .with(Modify::new(Columns::single(0)).with(Width::wrap(18).keep_words(true))) // Property column
        .with(Modify::new(Columns::single(1)).with(Width::wrap(60).keep_words(true))) // Value column
        .to_string();

    println!("{}", table);

    Ok(())
}

/// Add JSON fields to rows recursively
fn add_json_fields_to_rows(rows: &mut Vec<[String; 2]>, json_value: &Value, prefix: &str) {
    match json_value {
        Value::Object(map) => {
            for (key, value) in map {
                let display_key = if prefix.is_empty() {
                    format!("  {}", key)
                } else {
                    format!("  {}.{}", prefix, key)
                };

                match value {
                    Value::String(s) => {
                        // Skip very long text fields like full_text in display
                        if key == "full_text" && s.len() > 500 {
                            rows.push([
                                display_key,
                                format!("[{} characters - use search to find content]", s.len()),
                            ]);
                        } else if s.len() > 200 {
                            // Truncate other long strings
                            rows.push([display_key, format!("{}...", &s[..200])]);
                        } else {
                            rows.push([display_key, s.clone()]);
                        }
                    }
                    Value::Number(n) => {
                        rows.push([display_key, n.to_string()]);
                    }
                    Value::Bool(b) => {
                        let bool_str = if *b { "true" } else { "false" };
                        rows.push([display_key, bool_str.to_string()]);
                    }
                    Value::Array(arr) => {
                        let array_str = format!("[{} items]", arr.len());
                        rows.push([display_key, array_str]);

                        // Show first few items of array
                        for (i, item) in arr.iter().take(3).enumerate() {
                            let item_key = format!("    [{}]", i);
                            match item {
                                Value::String(s) => rows.push([item_key.to_string(), s.clone()]),
                                Value::Number(n) => {
                                    rows.push([item_key.to_string(), n.to_string()])
                                }
                                Value::Bool(b) => {
                                    let bool_str = if *b { "true" } else { "false" };
                                    rows.push([item_key.to_string(), bool_str.to_string()]);
                                }
                                _ => rows.push([item_key.to_string(), format!("{}", item)]),
                            }
                        }
                        if arr.len() > 3 {
                            rows.push(["    ...".to_string(), format!("({} more)", arr.len() - 3)]);
                        }
                    }
                    Value::Object(_) => {
                        rows.push([display_key.clone(), "[object]".to_string()]);
                        let new_prefix = if prefix.is_empty() {
                            key.clone()
                        } else {
                            format!("{}.{}", prefix, key)
                        };
                        add_json_fields_to_rows(rows, value, &new_prefix);
                    }
                    Value::Null => {
                        rows.push([display_key, "null".to_string()]);
                    }
                }
            }
        }
        _ => {
            // Handle non-object root values
            rows.push([format!("  {}", "value"), format!("{}", json_value)]);
        }
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
        "category1" => combine_categories(entry),
        "tags" => entry.tags.join(", "),
        "created_at" => entry.created_at.format("%Y-%m-%d %H:%M").to_string(),
        "updated_at" => entry.updated_at.format("%Y-%m-%d %H:%M").to_string(),
        _ => "-".to_string(),
    }
}

fn combine_categories(entry: &FileEntry) -> String {
    let mut categories = vec![entry.category1.clone()];
    if let Some(category2) = &entry.category2 {
        categories.push(category2.clone());
    }
    if let Some(category3) = &entry.category3 {
        categories.push(category3.clone());
    }

    categories.join("/")
}
