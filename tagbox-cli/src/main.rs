use clap::Parser;
use env_logger::Env;
use log::error;
use std::process;

mod cli;
mod commands;
mod output;
mod utils;

use cli::{AuthorCommands, Cli, Commands, ConfigCommands, DbCommands};
use utils::{config, error::CliError};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize logging
    if !cli.quiet {
        let env = Env::default().default_filter_or(&cli.log_level);
        env_logger::init_from_env(env);
    }

    // Handle commands that don't need existing config
    match &cli.command {
        Commands::InitConfig { force, output } => {
            let result = commands::init_config::handle_init_config(*force, output.as_deref()).await;
            if let Err(e) = result {
                error!("Command failed: {}", e);
                eprintln!("Error: {}", e);
                process::exit(1);
            }
            return;
        }
        Commands::Config { cd: true, .. } => {
            let result = commands::config::handle_config_cd().await;
            if let Err(e) = result {
                error!("Command failed: {}", e);
                eprintln!("Error: {}", e);
                process::exit(1);
            }
            return;
        }
        Commands::Db { command } => {
            // For db commands, we try to load config but don't fail if it doesn't exist
            let config = config::load_config(cli.config.as_deref())
                .await
                .unwrap_or_else(|_| {
                    // Use default config for db commands when config file doesn't exist
                    tagbox_core::config::AppConfig::default()
                });

            let result = commands::db::handle_db_command(command.clone(), &config).await;
            if let Err(e) = result {
                error!("Command failed: {}", e);
                eprintln!("Error: {}", e);
                process::exit(1);
            }
            return;
        }
        _ => {}
    }

    // Load configuration for all other commands
    let config = match config::load_config(cli.config.as_deref()).await {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            eprintln!();
            eprintln!("To create a new configuration file, run:");
            eprintln!("  tagbox init-config");
            eprintln!();
            eprintln!("Or specify an existing config file with:");
            eprintln!("  tagbox -c /path/to/config.toml <command>");
            eprintln!();
            eprintln!("Default config file locations (in order of precedence):");
            eprintln!("  - ./tagbox.toml (current directory)");
            if let Ok(default_config) = config::get_default_config_file() {
                eprintln!("  - {}", default_config.display());
            }

            // Show platform-specific paths
            if cfg!(target_os = "windows") {
                eprintln!("  - %APPDATA%\\tagbox\\tagbox.toml");
                eprintln!("  - %USERPROFILE%\\.config\\tagbox\\tagbox.toml");
                eprintln!("  - %USERPROFILE%\\.tagbox.toml");
            } else if cfg!(target_os = "macos") {
                eprintln!("  - ~/Library/Application Support/tagbox/tagbox.toml");
                eprintln!("  - ~/.config/tagbox/tagbox.toml");
                eprintln!("  - ~/.tagbox.toml");
                eprintln!("  - /etc/tagbox/tagbox.toml (system-wide)");
            } else {
                eprintln!("  - ~/.config/tagbox/tagbox.toml");
                eprintln!("  - ~/.tagbox.toml");
                eprintln!("  - /etc/tagbox/tagbox.toml (system-wide)");
            }
            process::exit(1);
        }
    };

    // Execute command
    let result = execute_command(cli.command, &config).await;

    if let Err(e) = result {
        error!("Command failed: {}", e);
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

async fn execute_command(
    command: Commands,
    config: &tagbox_core::config::AppConfig,
) -> Result<(), CliError> {
    // Check if database exists for commands that need it
    let needs_database = !matches!(command, Commands::InitConfig { .. } | Commands::Db { .. });

    if needs_database {
        if !commands::db::check_database_exists(config)
            .await
            .unwrap_or(false)
        {
            eprintln!("❌ Database not found or not properly initialized.");
            eprintln!("   Run 'tagbox db init' to create the database first.");
            eprintln!(
                "   Database should be at: {}",
                config.database.path.display()
            );
            return Err(CliError::DatabaseNotFound);
        }
    }

    match command {
        Commands::Import {
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
        } => {
            commands::import::handle_import(
                &path,
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
            .await
        }

        Commands::ImportUrl {
            url,
            rename,
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
        } => {
            commands::import::handle_import_url(
                &url,
                rename,
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
            .await
        }

        Commands::Search {
            query,
            json,
            columns,
            limit,
            offset,
        } => commands::search::handle_search(&query, json, columns, limit, offset, config).await,

        Commands::Preview {
            id,
            only_meta,
            open,
            cd,
        } => commands::preview::handle_preview(&id, only_meta, open, cd, config).await,

        Commands::Link { id1, id2, relation } => {
            commands::link::handle_link(&id1, &id2, relation, config).await
        }

        Commands::Unlink {
            id1,
            id2,
            batch,
            ids,
        } => commands::link::handle_unlink(&id1, &id2, batch, ids.as_deref(), config).await,

        Commands::QueryDebug { dsl } => commands::search::handle_query_debug(&dsl, config).await,

        Commands::Author { command } => match command {
            Some(AuthorCommands::Add { name }) => {
                commands::author::handle_author_add(&name, config).await
            }
            Some(AuthorCommands::Remove { id }) => {
                commands::author::handle_author_remove(&id, config).await
            }
            Some(AuthorCommands::Merge { from, to }) => {
                commands::author::handle_author_merge(&from, &to, config).await
            }
            None => commands::author::handle_author_list(config).await,
        },

        Commands::Config { cd, command } => {
            // cd case is handled above
            if cd {
                unreachable!("Config --cd should be handled before this match")
            } else {
                match command {
                    Some(ConfigCommands::Get { key }) => {
                        commands::config::handle_config_get(&key, config).await
                    }
                    Some(ConfigCommands::Set { key, value }) => {
                        commands::config::handle_config_set(&key, &value, config).await
                    }
                    Some(ConfigCommands::List) => {
                        commands::config::handle_config_list(config).await
                    }
                    None => commands::config::handle_config_list(config).await,
                }
            }
        }

        Commands::Export { json, output } => {
            commands::export::handle_export(json, output.as_deref(), config).await
        }

        Commands::Stats => commands::stats::handle_stats(config).await,

        Commands::Serve { port, host } => commands::serve::handle_serve(port, &host, config).await,

        Commands::Stdio => commands::stdio::handle_stdio(config).await,

        Commands::InitConfig { .. } => {
            // This case is handled above, but we need it here for exhaustive matching
            unreachable!("InitConfig should be handled before this match")
        }

        Commands::Db { .. } => {
            // This case is handled above, but we need it here for exhaustive matching
            unreachable!("Db commands should be handled before this match")
        }
    }
}
