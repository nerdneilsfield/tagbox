use crate::utils::error::{CliError, Result};
use std::path::{Path, PathBuf};
use tagbox_core::config::AppConfig;

/// Load configuration from default location or specified path
pub async fn load_config(config_path: Option<&Path>) -> Result<AppConfig> {
    let config_file = match config_path {
        Some(path) => path.to_path_buf(),
        None => find_config_file()?,
    };

    if !config_file.exists() {
        return Err(CliError::FileNotFound(format!(
            "Configuration file not found: {}",
            config_file.display()
        )));
    }

    tagbox_core::load_config(&config_file)
        .await
        .map_err(CliError::Core)
}

/// Find configuration file in standard locations
pub fn find_config_file() -> Result<PathBuf> {
    // 1. Try current directory first
    let current_dir_config = PathBuf::from("tagbox.toml");
    if current_dir_config.exists() {
        log::debug!(
            "Using config file from current directory: {}",
            current_dir_config.display()
        );
        return Ok(current_dir_config);
    }

    // 2. Try platform-specific config directory
    // Windows: %APPDATA%\tagbox\tagbox.toml
    // macOS: ~/Library/Application Support/tagbox/tagbox.toml
    // Linux: ~/.config/tagbox/tagbox.toml
    if let Some(config_dir) = dirs::config_dir() {
        let platform_config = config_dir.join("tagbox").join("tagbox.toml");
        if platform_config.exists() {
            log::debug!(
                "Using config file from platform config directory: {}",
                platform_config.display()
            );
            return Ok(platform_config);
        }
    }

    // 3. Try home directory .config (fallback for XDG)
    if let Some(home_dir) = dirs::home_dir() {
        let home_config = home_dir.join(".config").join("tagbox").join("tagbox.toml");
        if home_config.exists() {
            log::debug!(
                "Using config file from home .config: {}",
                home_config.display()
            );
            return Ok(home_config);
        }

        // 4. Try dotfile in home directory
        let dotfile_config = home_dir.join(".tagbox.toml");
        if dotfile_config.exists() {
            log::debug!(
                "Using config file from home dotfile: {}",
                dotfile_config.display()
            );
            return Ok(dotfile_config);
        }
    }

    // 5. Try system-wide config (Unix-like systems)
    let system_config = PathBuf::from("/etc/tagbox/tagbox.toml");
    if system_config.exists() {
        log::debug!("Using system-wide config file: {}", system_config.display());
        return Ok(system_config);
    }

    // 6. Default fallback - prefer platform-specific location for new installs
    let default_config = if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("tagbox").join("tagbox.toml")
    } else if let Some(home_dir) = dirs::home_dir() {
        home_dir.join(".config").join("tagbox").join("tagbox.toml")
    } else {
        PathBuf::from("tagbox.toml")
    };

    log::debug!(
        "No existing config found, will use default location: {}",
        default_config.display()
    );
    Ok(default_config)
}

/// Get the default configuration directory
pub fn get_default_config_dir() -> Result<PathBuf> {
    if let Some(config_dir) = dirs::config_dir() {
        Ok(config_dir.join("tagbox"))
    } else if let Some(home_dir) = dirs::home_dir() {
        Ok(home_dir.join(".config").join("tagbox"))
    } else {
        Ok(PathBuf::from("."))
    }
}

/// Get the default configuration file path
pub fn get_default_config_file() -> Result<PathBuf> {
    Ok(get_default_config_dir()?.join("tagbox.toml"))
}
