use crate::utils::error::{CliError, Result};
use std::path::Path;
use tagbox_core::config::AppConfig;

/// Handle link command
pub async fn handle_link(
    id1: &str,
    id2: &str,
    relation: Option<String>,
    config: &AppConfig,
) -> Result<()> {
    log::debug!(
        "Linking files: {} -> {} (relation: {:?})",
        id1,
        id2,
        relation
    );

    // Verify both files exist
    let _file1 = tagbox_core::get_file(id1, config).await?;
    let _file2 = tagbox_core::get_file(id2, config).await?;

    // Create the link
    tagbox_core::link_files(id1, id2, relation.clone(), config).await?;

    let relation_str = relation.unwrap_or_else(|| "relates".to_string());
    println!("Successfully linked {} -> {} ({})", id1, id2, relation_str);

    Ok(())
}

/// Handle unlink command
pub async fn handle_unlink(
    id1: &str,
    id2: &str,
    batch: bool,
    ids_file: Option<&Path>,
    config: &AppConfig,
) -> Result<()> {
    if batch && ids_file.is_some() {
        handle_batch_unlink(ids_file.unwrap(), config).await
    } else {
        handle_single_unlink(id1, id2, config).await
    }
}

/// Handle single unlink operation
async fn handle_single_unlink(id1: &str, id2: &str, config: &AppConfig) -> Result<()> {
    log::debug!("Unlinking files: {} -> {}", id1, id2);

    // Verify both files exist
    let _file1 = tagbox_core::get_file(id1, config).await?;
    let _file2 = tagbox_core::get_file(id2, config).await?;

    // Remove the link
    tagbox_core::unlink_files(id1, id2, config).await?;

    println!("Successfully unlinked {} -> {}", id1, id2);
    Ok(())
}

/// Handle batch unlink operation
async fn handle_batch_unlink(ids_file: &Path, config: &AppConfig) -> Result<()> {
    log::info!("Batch unlinking from file: {}", ids_file.display());

    let content = std::fs::read_to_string(ids_file)?;
    let mut success_count = 0;
    let mut error_count = 0;

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue; // Skip empty lines and comments
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 2 {
            eprintln!("Warning: Invalid format at line {}: {}", line_num + 1, line);
            error_count += 1;
            continue;
        }

        let id1 = parts[0];
        let id2 = parts[1];

        match tagbox_core::unlink_files(id1, id2, config).await {
            Ok(()) => {
                println!("Unlinked: {} -> {}", id1, id2);
                success_count += 1;
            }
            Err(e) => {
                eprintln!("Failed to unlink {} -> {}: {}", id1, id2, e);
                error_count += 1;
            }
        }
    }

    println!(
        "Batch unlink completed: {} successful, {} failed",
        success_count, error_count
    );

    if error_count > 0 {
        Err(CliError::CommandFailed(format!(
            "{} unlink operations failed",
            error_count
        )))
    } else {
        Ok(())
    }
}
