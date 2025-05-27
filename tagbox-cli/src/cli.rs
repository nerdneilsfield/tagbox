use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "tagbox")]
#[command(about = "A CLI tool to manage file metadata using local SQLite + full-text search")]
#[command(version)]
pub struct Cli {
    /// Configuration file path
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    /// Log level (info, warn, debug)
    #[arg(long, default_value = "info")]
    pub log_level: String,

    /// Suppress normal output (overrides log level)
    #[arg(long)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Import a file or directory of files
    Import {
        /// Path to file or directory
        path: PathBuf,

        /// Delete original after import (copy and delete)
        #[arg(short, long)]
        delete: bool,

        /// Specify the category for file to store (relative path of storage_path)
        #[arg(short, long)]
        category: Option<String>,

        /// Use id to specify the category
        #[arg(long)]
        category_id: Option<String>,

        /// Specify the title of the file
        #[arg(long)]
        title: Option<String>,

        /// Specify the authors (comma-separated)
        #[arg(long)]
        authors: Option<String>,

        /// Specify the year
        #[arg(long)]
        year: Option<i32>,

        /// Specify the publisher
        #[arg(long)]
        publisher: Option<String>,

        /// Specify the source
        #[arg(long)]
        source: Option<String>,

        /// Specify tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,

        /// Specify summary
        #[arg(long)]
        summary: Option<String>,

        /// JSON file to set file attributes
        #[arg(long)]
        meta_file: Option<PathBuf>,
    },

    /// Download and import a file from a URL
    ImportUrl {
        /// URL to download from
        url: String,

        /// Override filename
        #[arg(long)]
        rename: Option<String>,

        /// Delete original after import (copy and delete)
        #[arg(short, long)]
        delete: bool,

        /// Specify the category for file to store (relative path of storage_path)
        #[arg(short, long)]
        category: Option<String>,

        /// Use id to specify the category
        #[arg(long)]
        category_id: Option<String>,

        /// Specify the title of the file
        #[arg(long)]
        title: Option<String>,

        /// Specify the authors (comma-separated)
        #[arg(long)]
        authors: Option<String>,

        /// Specify the year
        #[arg(long)]
        year: Option<i32>,

        /// Specify the publisher
        #[arg(long)]
        publisher: Option<String>,

        /// Specify the source
        #[arg(long)]
        source: Option<String>,

        /// Specify tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,

        /// Specify summary
        #[arg(long)]
        summary: Option<String>,

        /// JSON file to set file attributes
        #[arg(long)]
        meta_file: Option<PathBuf>,
    },

    /// Search files using DSL or free text
    Search {
        /// Search query (DSL or free text)
        query: String,

        /// Output result as JSON
        #[arg(long)]
        json: bool,

        /// Comma-separated fields (e.g., title,path,authors)
        #[arg(long)]
        columns: Option<String>,

        /// Maximum number of results
        #[arg(long)]
        limit: Option<usize>,

        /// Offset for pagination
        #[arg(long)]
        offset: Option<usize>,
    },

    /// Show a file's metadata
    Preview {
        /// File ID
        id: String,

        /// Only show metadata, no summary or path
        #[arg(long)]
        only_meta: bool,

        /// Open file with system default program
        #[arg(long)]
        open: bool,

        /// Print path to containing folder
        #[arg(long)]
        cd: bool,
    },

    /// Link two files as semantically related
    Link {
        /// First file ID
        id1: String,

        /// Second file ID
        id2: String,

        /// Optional relation type (e.g., reference)
        #[arg(long)]
        relation: Option<String>,
    },

    /// Remove semantic link between two files
    Unlink {
        /// First file ID
        id1: String,

        /// Second file ID
        id2: String,

        /// Unlink many (batch mode)
        #[arg(long)]
        batch: bool,

        /// File of ID pairs
        #[arg(long)]
        ids: Option<PathBuf>,
    },

    /// Show SQL generated from DSL query and count preview
    QueryDebug {
        /// DSL query
        dsl: String,
    },

    /// Manage author entries
    Author {
        #[command(subcommand)]
        command: Option<AuthorCommands>,
    },

    /// Edit runtime parameters
    Config {
        /// Print path to config file directory (for shell integration)
        #[arg(long)]
        cd: bool,

        #[command(subcommand)]
        command: Option<ConfigCommands>,
    },

    /// Dump files in JSON or CSV format
    Export {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Show tag usage, top authors, access heatmap
    Stats,

    /// Launch local MCP-compatible server endpoint
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },

    /// JSON-RPC mode for external integrations
    Stdio,

    /// Initialize configuration file
    InitConfig {
        /// Force overwrite existing config
        #[arg(long)]
        force: bool,

        /// Custom config file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum AuthorCommands {
    /// Add a new author
    Add {
        /// Author name
        name: String,
    },

    /// Remove an author
    Remove {
        /// Author ID
        id: String,
    },

    /// Merge authors
    Merge {
        /// Source author ID
        from: String,

        /// Target author ID
        to: String,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Get configuration value
    Get {
        /// Configuration key
        key: String,
    },

    /// Set configuration value
    Set {
        /// Configuration key
        key: String,

        /// Configuration value
        value: String,
    },

    /// List all configuration values
    List,
}
