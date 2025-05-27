use crate::utils::error::{CliError, Result};
use tagbox_core::config::AppConfig;

/// Handle config get command
pub async fn handle_config_get(key: &str, config: &AppConfig) -> Result<()> {
    log::debug!("Getting config value for key: {}", key);

    let value = get_config_value(config, key)?;
    println!("{}", value);

    Ok(())
}

/// Handle config set command
pub async fn handle_config_set(key: &str, value: &str, _config: &AppConfig) -> Result<()> {
    log::debug!("Setting config value: {} = {}", key, value);

    // For now, just show what would be set
    // TODO: Implement actual config modification when core supports it
    println!("Config setting not yet implemented in core.");
    println!("Would set: {} = {}", key, value);

    Ok(())
}

/// Get configuration value by dotted key path
fn get_config_value(config: &AppConfig, key: &str) -> Result<String> {
    let parts: Vec<&str> = key.split('.').collect();

    match parts.as_slice() {
        ["database", "path"] => Ok(config.database.path.to_string_lossy().to_string()),
        ["database", "journal_mode"] => Ok(config.database.journal_mode.to_string()),
        ["database", "sync_mode"] => Ok(config.database.sync_mode.to_string()),

        ["import", "paths", "storage_dir"] => Ok(config
            .import
            .paths
            .storage_dir
            .to_string_lossy()
            .to_string()),
        ["import", "paths", "rename_template"] => {
            Ok(config.import.paths.rename_template.to_string())
        }
        ["import", "paths", "classify_template"] => {
            Ok(config.import.paths.classify_template.to_string())
        }
        ["import", "metadata", "prefer_json"] => Ok(config.import.metadata.prefer_json.to_string()),
        ["import", "metadata", "fallback_pdf"] => {
            Ok(config.import.metadata.fallback_pdf.to_string())
        }
        ["import", "metadata", "default_category"] => {
            Ok(config.import.metadata.default_category.to_string())
        }

        ["search", "default_limit"] => Ok(config.search.default_limit.to_string()),
        ["search", "enable_fts"] => Ok(config.search.enable_fts.to_string()),
        ["search", "fts_language"] => Ok(config.search.fts_language.to_string()),

        ["hash", "algorithm"] => Ok(config.hash.algorithm.to_string()),
        ["hash", "verify_on_import"] => Ok(config.hash.verify_on_import.to_string()),

        _ => {
            // Try to find partial matches and suggest
            let available_keys = get_available_config_keys();
            let suggestions: Vec<_> = available_keys
                .iter()
                .filter(|k| k.contains(key))
                .take(5)
                .collect();

            let mut error_msg = format!("Unknown config key: {}", key);
            if !suggestions.is_empty() {
                error_msg.push_str("\nDid you mean one of these?\n");
                for suggestion in suggestions {
                    error_msg.push_str(&format!("  {}\n", suggestion));
                }
            }

            Err(CliError::InvalidArgument(error_msg))
        }
    }
}

/// Get list of all available configuration keys
fn get_available_config_keys() -> Vec<String> {
    vec![
        "database.path".to_string(),
        "database.journal_mode".to_string(),
        "database.sync_mode".to_string(),
        "import.paths.storage_dir".to_string(),
        "import.paths.rename_template".to_string(),
        "import.paths.classify_template".to_string(),
        "import.metadata.prefer_json".to_string(),
        "import.metadata.fallback_pdf".to_string(),
        "import.metadata.default_category".to_string(),
        "search.default_limit".to_string(),
        "search.enable_fts".to_string(),
        "search.fts_language".to_string(),
        "hash.algorithm".to_string(),
        "hash.verify_on_import".to_string(),
    ]
}

/// Handle config cd command
pub async fn handle_config_cd() -> Result<()> {
    log::debug!("Getting config directory path");

    let config_file = crate::utils::config::find_config_file()?;
    if let Some(parent) = config_file.parent() {
        println!("{}", parent.display());
    } else {
        println!(".");
    }

    Ok(())
}

/// List all configuration values
pub async fn handle_config_list(config: &AppConfig) -> Result<()> {
    log::info!("Listing all configuration values");

    println!("Current Configuration:");
    println!("=====================");
    println!();

    println!("[database]");
    println!("path = {}", config.database.path.display());
    println!("journal_mode = {}", config.database.journal_mode);
    println!("sync_mode = {}", config.database.sync_mode);
    println!();

    println!("[import.paths]");
    println!(
        "storage_dir = {}",
        config.import.paths.storage_dir.display()
    );
    println!("rename_template = {}", config.import.paths.rename_template);
    println!(
        "classify_template = {}",
        config.import.paths.classify_template
    );
    println!();

    println!("[import.metadata]");
    println!("prefer_json = {}", config.import.metadata.prefer_json);
    println!("fallback_pdf = {}", config.import.metadata.fallback_pdf);
    println!(
        "default_category = {}",
        config.import.metadata.default_category
    );
    println!();

    println!("[search]");
    println!("default_limit = {}", config.search.default_limit);
    println!("enable_fts = {}", config.search.enable_fts);
    println!("fts_language = {}", config.search.fts_language);
    println!();

    println!("[hash]");
    println!("algorithm = {}", config.hash.algorithm);
    println!("verify_on_import = {}", config.hash.verify_on_import);

    Ok(())
}
