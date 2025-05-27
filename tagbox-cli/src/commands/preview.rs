use crate::output::{json, table};
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
    log::debug!("Previewing file: {}", id);

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
        // Show formatted preview using table format
        table::print_preview_table(&file_entry)?;
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
