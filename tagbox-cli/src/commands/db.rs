use crate::cli::DbCommands;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tagbox_core::schema::Database;
use tokio::fs::{self, File};

pub async fn handle_db_command(
    command: DbCommands,
    config: &tagbox_core::config::AppConfig,
) -> Result<()> {
    match command {
        DbCommands::Init { force, path } => {
            let db_path = path.unwrap_or_else(|| config.database.path.clone());
            init_database(&db_path, force).await
        }
        DbCommands::Cd => {
            let db_path = &config.database.path;
            if let Some(parent) = db_path.parent() {
                println!("{}", parent.display());
            } else {
                println!(".");
            }
            Ok(())
        }
        DbCommands::Path => {
            println!("{}", config.database.path.display());
            Ok(())
        }
        DbCommands::Status => check_database_status(config).await,
    }
}

async fn init_database(db_path: &Path, force: bool) -> Result<()> {
    // Check if database already exists
    if db_path.exists() && !force {
        return Err(anyhow::anyhow!(
            "Database already exists at {}. Use --force to overwrite.",
            db_path.display()
        ));
    }

    // Create parent directory if it doesn't exist
    if let Some(parent) = db_path.parent() {
        println!("Creating directory: {}", parent.display());
        fs::create_dir_all(parent)
            .await
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }

    // Remove existing database if force is true
    if db_path.exists() && force {
        fs::remove_file(db_path)
            .await
            .with_context(|| format!("Failed to remove existing database {}", db_path.display()))?;
    }

    // Create empty database file first
    println!("Creating database file: {}", db_path.display());
    File::create(db_path)
        .await
        .with_context(|| format!("Failed to create database file {}", db_path.display()))?;

    // Create new database using Database struct
    println!("Initializing database connection: {}", db_path.display());
    let database = Database::new(db_path).await.map_err(|e| {
        anyhow::anyhow!(
            "Failed to initialize database at {}: {}",
            db_path.display(),
            e
        )
    })?;

    // Apply migrations to create tables
    database
        .migrate()
        .await
        .context("Failed to create database tables")?;

    println!("Database initialized successfully at {}", db_path.display());
    Ok(())
}

async fn check_database_status(config: &tagbox_core::config::AppConfig) -> Result<()> {
    let db_path = &config.database.path;

    if !db_path.exists() {
        println!("❌ Database does not exist at {}", db_path.display());
        println!("   Run 'tagbox db init' to create the database.");
        return Ok(());
    }

    // Try to connect to the database
    let db_url = format!("sqlite:{}", db_path.display());
    match sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await
    {
        Ok(pool) => {
            // Check if tables exist
            let table_check =
                sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='files'")
                    .fetch_optional(&pool)
                    .await;

            match table_check {
                Ok(Some(_)) => {
                    println!("✅ Database is ready at {}", db_path.display());

                    // Get file count
                    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM files")
                        .fetch_one(&pool)
                        .await
                        .unwrap_or((0,));

                    println!("   Contains {} files", count.0);
                }
                Ok(None) => {
                    println!(
                        "⚠️  Database exists but tables are missing at {}",
                        db_path.display()
                    );
                    println!("   Run 'tagbox db init --force' to recreate the database.");
                }
                Err(e) => {
                    println!("❌ Database exists but has errors: {}", e);
                    println!("   Run 'tagbox db init --force' to recreate the database.");
                }
            }
        }
        Err(e) => {
            println!(
                "❌ Cannot connect to database at {}: {}",
                db_path.display(),
                e
            );
            println!("   Run 'tagbox db init --force' to recreate the database.");
        }
    }

    Ok(())
}

pub async fn check_database_exists(config: &tagbox_core::config::AppConfig) -> Result<bool> {
    let db_path = &config.database.path;

    if !db_path.exists() {
        return Ok(false);
    }

    // Try to connect and check if tables exist
    let db_url = format!("sqlite:{}", db_path.display());
    match sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await
    {
        Ok(pool) => {
            let table_check =
                sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='files'")
                    .fetch_optional(&pool)
                    .await;

            match table_check {
                Ok(Some(_)) => Ok(true),
                _ => Ok(false),
            }
        }
        Err(_) => Ok(false),
    }
}
