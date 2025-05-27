use crate::output::json;
use crate::utils::error::{CliError, Result};
use std::process::Command;
use tagbox_core::config::AppConfig;

/// Handle preview command
pub async fn handle_preview(
    id: &str,
    only_meta: bool,
    open: bool,
    cd: bool,
    config: &AppConfig,
) -> Result<()> {
    log::info!("Previewing file: {}", id);

    let file_entry = tagbox_core::get_file(id, config).await?;

    if cd {
        // Print just the directory path for shell integration
        if let Some(parent) = file_entry.path.parent() {
            println!("{}", parent.display());
        } else {
            println!(".");
        }
        return Ok(());
    }

    if open {
        // Open the file with system default program
        open_file(&file_entry.path)?;
        println!("Opened file: {}", file_entry.path.display());
        return Ok(());
    }

    if only_meta {
        // Show only metadata in JSON format
        json::print_json(&file_entry)?;
    } else {
        // Show formatted preview
        print_file_preview(&file_entry)?;
    }

    Ok(())
}

/// Print formatted file preview
fn print_file_preview(entry: &tagbox_core::types::FileEntry) -> Result<()> {
    println!("File Preview");
    println!("============");
    println!();
    println!("ID: {}", entry.id);
    println!("Title: {}", entry.title);

    if !entry.authors.is_empty() {
        println!("Authors: {}", entry.authors.join(", "));
    }

    if let Some(year) = entry.year {
        println!("Year: {}", year);
    }

    if let Some(publisher) = &entry.publisher {
        println!("Publisher: {}", publisher);
    }

    if let Some(source) = &entry.source {
        println!("Source: {}", source);
    }

    println!("Category: {}", entry.category1);
    if let Some(cat2) = &entry.category2 {
        println!("Subcategory: {}", cat2);
    }
    if let Some(cat3) = &entry.category3 {
        println!("Sub-subcategory: {}", cat3);
    }

    if !entry.tags.is_empty() {
        println!("Tags: {}", entry.tags.join(", "));
    }

    if let Some(summary) = &entry.summary {
        println!("Summary: {}", summary);
    }

    println!();
    println!("File Information");
    println!("================");
    println!("Path: {}", entry.path.display());
    println!("Original filename: {}", entry.original_filename);
    println!("Hash: {}", entry.hash);

    if let Some(current_hash) = &entry.current_hash {
        if current_hash != &entry.hash {
            println!("Current hash: {} (CHANGED!)", current_hash);
        }
    }

    println!("Created: {}", entry.created_at.format("%Y-%m-%d %H:%M:%S"));
    println!("Updated: {}", entry.updated_at.format("%Y-%m-%d %H:%M:%S"));

    if let Some(accessed) = entry.last_accessed {
        println!("Last accessed: {}", accessed.format("%Y-%m-%d %H:%M:%S"));
    }

    if entry.is_deleted {
        println!("Status: DELETED");
    }

    Ok(())
}

/// Open file with system default program
fn open_file(path: &std::path::Path) -> Result<()> {
    let result = if cfg!(target_os = "macos") {
        Command::new("open").arg(path).status()
    } else if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(path)
            .status()
    } else {
        // Linux and other Unix-like systems
        Command::new("xdg-open").arg(path).status()
    };

    match result {
        Ok(status) if status.success() => Ok(()),
        Ok(_) => Err(CliError::CommandFailed(
            "Failed to open file with default program".to_string(),
        )),
        Err(e) => Err(CliError::Io(e)),
    }
}
