use crate::utils::{config, error::Result};
use std::path::Path;

/// Handle init-config command
pub async fn handle_init_config(force: bool, output: Option<&Path>) -> Result<()> {
    log::info!("Initializing configuration file");

    // Determine output path
    let config_path = match output {
        Some(path) => path.to_path_buf(),
        None => config::get_default_config_file()?,
    };

    // Check if file already exists
    if config_path.exists() && !force {
        eprintln!(
            "Configuration file already exists at: {}",
            config_path.display()
        );
        eprintln!("Use --force to overwrite or specify a different path with --output");
        return Ok(());
    }

    // Create directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Generate default configuration
    let default_config = generate_default_config(&config_path)?;

    // Write configuration file
    std::fs::write(&config_path, default_config)?;

    println!(
        "✅ Configuration file created at: {}",
        config_path.display()
    );
    println!();
    println!("Next steps:");
    println!("  1. Review and edit the configuration file as needed");
    println!("  2. Initialize the database: cargo run --bin tagbox-init-db");
    println!("  3. Start importing files: tagbox import <path>");
    println!();
    println!("Configuration help:");
    println!("  - View current config: tagbox config list");
    println!("  - Get specific value: tagbox config get <key>");
    println!("  - Get config directory: tagbox config --cd");
    println!("  - Edit file directly: $EDITOR {}", config_path.display());
    println!();

    // Show platform-specific shell integration examples
    if cfg!(target_os = "windows") {
        println!("Shell integration examples (Windows):");
        println!("  cd /D \"$(tagbox config --cd)\"");
        println!("  explorer \"$(tagbox config --cd)\"");
    } else {
        println!("Shell integration examples (Unix/macOS):");
        println!("  cd \"$(tagbox config --cd)\"");
        println!("  open \"$(tagbox config --cd)\"  # macOS");
        println!("  xdg-open \"$(tagbox config --cd)\"  # Linux");
    }

    Ok(())
}

/// Generate default configuration content
fn generate_default_config(config_path: &Path) -> Result<String> {
    // Get reasonable default paths based on config location
    let config_dir = config_path.parent().unwrap_or_else(|| Path::new("."));

    let storage_dir = config_dir.join("storage");
    let database_path = config_dir.join("tagbox.db");

    let config_content = format!(
        r#"# TagBox Configuration File
# This file configures the TagBox file management system

[import]
[import.paths]
# Directory where imported files will be stored
storage_dir = "{}"

# Template for renaming files during import
# Available variables: {{title}}, {{year}}, {{author}}, {{category}}, {{original_name}}
rename_template = "{{title}}_{{year}}"

# Template for organizing files into subdirectories
# Available variables: {{category1}}, {{category2}}, {{author}}, {{year}}, {{filename}}
classify_template = "{{category1}}/{{filename}}"

[import.metadata]
# Prefer JSON metadata files when available
prefer_json = true

# Fallback to PDF metadata extraction if JSON is not available
fallback_pdf = true

# Default category for files without explicit category
default_category = "uncategorized"

[search]
# Default number of search results to return
default_limit = 50

# Enable full-text search (FTS5)
enable_fts = true

# Language for FTS tokenizer (simple, unicode61, porter, etc.)
fts_language = "unicode61"

[database]
# Path to SQLite database file
path = "{}"

# SQLite journal mode (DELETE, TRUNCATE, PERSIST, MEMORY, WAL, OFF)
journal_mode = "WAL"

# SQLite synchronous mode (OFF, NORMAL, FULL, EXTRA)
sync_mode = "NORMAL"

[hash]
# Hash algorithm for file integrity checking (blake2b, sha256)
algorithm = "blake2b"

# Verify file hashes on import (slower but safer)
verify_on_import = true
"#,
        storage_dir.display(),
        database_path.display()
    );

    // On Windows, escape backslashes for TOML after all template substitution
    let final_config = if cfg!(target_os = "windows") {
        config_content.replace('\\', "\\\\")
    } else {
        config_content
    };

    Ok(final_config)
}
