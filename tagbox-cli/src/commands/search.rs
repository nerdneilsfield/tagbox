use crate::output::{json, table};
use crate::utils::error::Result;
use tagbox_core::config::AppConfig;
use tagbox_core::types::SearchOptions;

/// Handle search command
pub async fn handle_search(
    query: &str,
    json_output: bool,
    columns: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    config: &AppConfig,
) -> Result<()> {
    log::debug!("Searching for: {}", query);

    let search_options = Some(SearchOptions {
        offset: offset.unwrap_or(0),
        limit: limit.unwrap_or(50),
        sort_by: None,
        sort_direction: None,
        include_deleted: false,
    });

    let result = tagbox_core::search_files_advanced(query, search_options, config).await?;

    if json_output {
        json::print_json(&result)?;
    } else {
        println!(
            "Found {} results (showing {}-{} of {})",
            result.entries.len(),
            result.offset + 1,
            result.offset + result.entries.len(),
            result.total_count
        );

        table::print_file_table(&result.entries, columns.as_deref())?;
    }

    Ok(())
}

/// Handle query debug command
pub async fn handle_query_debug(dsl: &str, config: &AppConfig) -> Result<()> {
    log::debug!("Debugging DSL query: {}", dsl);

    // Create a searcher to debug the query
    let db = tagbox_core::schema::Database::new(&config.database.path).await?;
    let searcher = tagbox_core::Searcher::new(config.clone(), db.pool().clone()).await;

    // For now, just run the query and show results count
    // TODO: Add actual SQL debug output when available in core
    let result = searcher.search_advanced(dsl, None).await?;

    println!("DSL Query: {}", dsl);
    println!("Results found: {}", result.total_count);
    println!("Query executed successfully");

    if result.total_count > 0 {
        println!("\nFirst few results:");
        let preview_entries = result.entries.iter().take(3).collect::<Vec<_>>();
        for entry in preview_entries {
            println!("  {} - {}", entry.id, entry.title);
        }

        if result.total_count > 3 {
            println!("  ... and {} more", result.total_count - 3);
        }
    }

    Ok(())
}
