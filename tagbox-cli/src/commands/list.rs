use crate::output::{json, table};
use crate::utils::error::Result;
use tagbox_core::config::AppConfig;
use tagbox_core::types::SearchOptions;

/// Handle list command
pub async fn handle_list(
    json_output: bool,
    columns: Option<String>,
    per_page: usize,
    page: usize,
    sort_by: &str,
    asc: bool,
    config: &AppConfig,
) -> Result<()> {
    let direction = if asc { "ASC" } else { "DESC" };
    log::info!(
        "Listing files: page {}, {} per page, sort by {} {}",
        page,
        per_page,
        sort_by,
        direction
    );

    // Calculate offset from page number (page starts from 1)
    let offset = if page > 0 { (page - 1) * per_page } else { 0 };

    // Create search options for pagination and sorting
    let search_options = Some(SearchOptions {
        offset,
        limit: per_page,
        sort_by: Some(sort_by.to_string()),
        sort_direction: Some(direction.to_string()),
        include_deleted: false,
    });

    // Use wildcard search to get all files with sorting and pagination
    let result = tagbox_core::search_files_advanced("*", search_options, config).await?;

    if json_output {
        json::print_json(&result)?;
    } else {
        // Calculate total pages
        let total_pages = (result.total_count + per_page - 1) / per_page;

        println!(
            "📄 Page {} of {} (showing {}-{} of {} files)",
            page,
            total_pages,
            result.offset + 1,
            result.offset + result.entries.len(),
            result.total_count
        );
        println!("📊 Sorted by: {} ({})", sort_by, direction);
        println!();

        table::print_file_table(&result.entries, columns.as_deref())?;

        // Show pagination info
        if total_pages > 1 {
            println!();
            if page > 1 {
                println!("⬅️  Previous: tagbox list --page {}", page - 1);
            }
            if page < total_pages {
                println!("➡️  Next: tagbox list --page {}", page + 1);
            }
        }
    }

    Ok(())
}
