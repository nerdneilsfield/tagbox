use crate::utils::error::Result;
use std::collections::HashMap;
use tagbox_core::config::AppConfig;
use tagbox_core::types::{FileEntry, SearchOptions};

/// Handle stats command
pub async fn handle_stats(config: &AppConfig) -> Result<()> {
    log::info!("Generating statistics");

    // Get all files for analysis
    let search_options = Some(SearchOptions {
        offset: 0,
        limit: 1000000,
        sort_by: None,
        sort_direction: None,
        include_deleted: false,
    });

    let result = tagbox_core::search_files_advanced("*", search_options, config).await?;
    let files = &result.entries;

    if files.is_empty() {
        println!("No files found in database.");
        return Ok(());
    }

    print_general_stats(files)?;
    print_tag_stats(files)?;
    print_author_stats(files)?;
    print_category_stats(files)?;
    print_year_stats(files)?;

    // Try to get access stats from core if available
    if let Ok(access_stats) = get_access_stats(config).await {
        print_access_stats(&access_stats)?;
    }

    Ok(())
}

/// Print general statistics
fn print_general_stats(files: &[FileEntry]) -> Result<()> {
    println!("General Statistics");
    println!("==================");
    println!("Total files: {}", files.len());

    let deleted_count = files.iter().filter(|f| f.is_deleted).count();
    println!("Active files: {}", files.len() - deleted_count);
    println!("Deleted files: {}", deleted_count);

    if let Some(oldest) = files.iter().min_by_key(|f| f.created_at) {
        println!(
            "Oldest file: {} ({})",
            oldest.title,
            oldest.created_at.format("%Y-%m-%d")
        );
    }

    if let Some(newest) = files.iter().max_by_key(|f| f.created_at) {
        println!(
            "Newest file: {} ({})",
            newest.title,
            newest.created_at.format("%Y-%m-%d")
        );
    }

    println!();
    Ok(())
}

/// Print tag usage statistics
fn print_tag_stats(files: &[FileEntry]) -> Result<()> {
    let mut tag_counts: HashMap<String, usize> = HashMap::new();

    for file in files {
        if file.is_deleted {
            continue;
        }

        for tag in &file.tags {
            *tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }
    }

    if tag_counts.is_empty() {
        println!("Tag Statistics: No tags found");
        println!();
        return Ok(());
    }

    let mut sorted_tags: Vec<_> = tag_counts.into_iter().collect();
    sorted_tags.sort_by(|a, b| b.1.cmp(&a.1));

    println!("Tag Statistics (Top 10)");
    println!("=======================");

    for (tag, count) in sorted_tags.iter().take(10) {
        println!("{:<20} {}", tag, count);
    }

    println!("Total unique tags: {}", sorted_tags.len());
    println!();
    Ok(())
}

/// Print author statistics
fn print_author_stats(files: &[FileEntry]) -> Result<()> {
    let mut author_counts: HashMap<String, usize> = HashMap::new();

    for file in files {
        if file.is_deleted {
            continue;
        }

        for author in &file.authors {
            *author_counts.entry(author.clone()).or_insert(0) += 1;
        }
    }

    if author_counts.is_empty() {
        println!("Author Statistics: No authors found");
        println!();
        return Ok(());
    }

    let mut sorted_authors: Vec<_> = author_counts.into_iter().collect();
    sorted_authors.sort_by(|a, b| b.1.cmp(&a.1));

    println!("Author Statistics (Top 10)");
    println!("==========================");

    for (author, count) in sorted_authors.iter().take(10) {
        println!("{:<30} {}", author, count);
    }

    println!("Total unique authors: {}", sorted_authors.len());
    println!();
    Ok(())
}

/// Print category statistics
fn print_category_stats(files: &[FileEntry]) -> Result<()> {
    let mut category_counts: HashMap<String, usize> = HashMap::new();

    for file in files {
        if file.is_deleted {
            continue;
        }

        *category_counts.entry(file.category1.clone()).or_insert(0) += 1;
    }

    let mut sorted_categories: Vec<_> = category_counts.into_iter().collect();
    sorted_categories.sort_by(|a, b| b.1.cmp(&a.1));

    println!("Category Statistics");
    println!("==================");

    for (category, count) in sorted_categories {
        println!("{:<20} {}", category, count);
    }

    println!();
    Ok(())
}

/// Print year statistics
fn print_year_stats(files: &[FileEntry]) -> Result<()> {
    let mut year_counts: HashMap<i32, usize> = HashMap::new();
    let mut no_year_count = 0;

    for file in files {
        if file.is_deleted {
            continue;
        }

        if let Some(year) = file.year {
            *year_counts.entry(year).or_insert(0) += 1;
        } else {
            no_year_count += 1;
        }
    }

    if year_counts.is_empty() && no_year_count == 0 {
        return Ok(());
    }

    let mut sorted_years: Vec<_> = year_counts.into_iter().collect();
    sorted_years.sort_by(|a, b| b.0.cmp(&a.0)); // Sort by year descending

    println!("Year Statistics (Last 10 years)");
    println!("===============================");

    for (year, count) in sorted_years.iter().take(10) {
        println!("{:<10} {}", year, count);
    }

    if no_year_count > 0 {
        println!("{:<10} {}", "Unknown", no_year_count);
    }

    println!();
    Ok(())
}

/// Get access statistics from core
async fn get_access_stats(
    config: &AppConfig,
) -> Result<Vec<tagbox_core::history::FileAccessStatsEntry>> {
    tagbox_core::get_most_accessed_files(10, config)
        .await
        .map_err(Into::into)
}

/// Print access statistics
fn print_access_stats(stats: &[tagbox_core::history::FileAccessStatsEntry]) -> Result<()> {
    if stats.is_empty() {
        return Ok(());
    }

    println!("Access Statistics (Top 10)");
    println!("==========================");

    for (i, stat) in stats.iter().enumerate() {
        println!(
            "{}. {} (accessed {} times)",
            i + 1,
            &stat.file_id[..8.min(stat.file_id.len())],
            stat.access_count
        );
    }

    println!();
    Ok(())
}
