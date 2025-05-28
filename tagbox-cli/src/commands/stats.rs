use crate::utils::error::Result;
// removed: use colored::*;
use std::collections::HashMap;
use tabled::{
    builder::Builder,
    settings::{object::Columns, Modify, Style, Width},
};
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

    let mut builder = Builder::default();
    builder.push_record(["Metric", "Value"]);

    let deleted_count = files.iter().filter(|f| f.is_deleted).count();
    let active_count = files.len() - deleted_count;

    builder.push_record(["Total files", &files.len().to_string()]);
    builder.push_record(["Active files", &active_count.to_string()]);
    builder.push_record(["Deleted files", &deleted_count.to_string()]);

    if let Some(oldest) = files.iter().min_by_key(|f| f.created_at) {
        let oldest_info = format!(
            "{} ({})",
            oldest.title.chars().take(30).collect::<String>(),
            oldest.created_at.format("%Y-%m-%d").to_string()
        );
        builder.push_record(["Oldest file", &oldest_info]);
    }

    if let Some(newest) = files.iter().max_by_key(|f| f.created_at) {
        let newest_info = format!(
            "{} ({})",
            newest.title.chars().take(30).collect::<String>(),
            newest.created_at.format("%Y-%m-%d").to_string()
        );
        builder.push_record(["Newest file", &newest_info]);
    }

    let table = builder
        .build()
        .with(Style::rounded())
        .with(Modify::new(Columns::single(0)).with(Width::wrap(20).keep_words(true)))
        .with(Modify::new(Columns::single(1)).with(Width::wrap(50).keep_words(true)))
        .to_string();

    println!("{}", table);
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
        println!("No tags found");
        println!();
        return Ok(());
    }

    let mut sorted_tags: Vec<_> = tag_counts.into_iter().collect();
    sorted_tags.sort_by(|a, b| b.1.cmp(&a.1));

    println!("Tag Statistics (Top 10)");

    let mut builder = Builder::default();
    builder.push_record(["Tag", "Count"]);

    for (tag, count) in sorted_tags.iter().take(10) {
        builder.push_record([tag.to_string(), count.to_string()]);
    }

    let table = builder
        .build()
        .with(Style::rounded())
        .with(Modify::new(Columns::single(0)).with(Width::wrap(25).keep_words(true)))
        .with(Modify::new(Columns::single(1)).with(Width::wrap(10).keep_words(true)))
        .to_string();

    println!("{}", table);
    println!("Total unique tags: {}", sorted_tags.len().to_string());
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
        println!("No authors found");
        println!();
        return Ok(());
    }

    let mut sorted_authors: Vec<_> = author_counts.into_iter().collect();
    sorted_authors.sort_by(|a, b| b.1.cmp(&a.1));

    println!("Author Statistics (Top 10)");

    let mut builder = Builder::default();
    builder.push_record(["Author", "Files"]);

    for (author, count) in sorted_authors.iter().take(10) {
        builder.push_record([author.to_string(), count.to_string()]);
    }

    let table = builder
        .build()
        .with(Style::rounded())
        .with(Modify::new(Columns::single(0)).with(Width::wrap(30).keep_words(true)))
        .with(Modify::new(Columns::single(1)).with(Width::wrap(10).keep_words(true)))
        .to_string();

    println!("{}", table);
    println!("Total unique authors: {}", sorted_authors.len().to_string());
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

    let mut builder = Builder::default();
    builder.push_record(["Category", "Files"]);

    for (category, count) in sorted_categories {
        let category_display = category.to_string();
        builder.push_record([category_display, count.to_string()]);
    }

    let table = builder
        .build()
        .with(Style::rounded())
        .with(Modify::new(Columns::single(0)).with(Width::wrap(25).keep_words(true)))
        .with(Modify::new(Columns::single(1)).with(Width::wrap(10).keep_words(true)))
        .to_string();

    println!("{}", table);
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

    let mut builder = Builder::default();
    builder.push_record(["Year", "Files"]);

    for (year, count) in sorted_years.iter().take(10) {
        builder.push_record([year.to_string(), count.to_string()]);
    }

    if no_year_count > 0 {
        builder.push_record(["Unknown".to_string(), no_year_count.to_string()]);
    }

    let table = builder
        .build()
        .with(Style::rounded())
        .with(Modify::new(Columns::single(0)).with(Width::wrap(15).keep_words(true)))
        .with(Modify::new(Columns::single(1)).with(Width::wrap(10).keep_words(true)))
        .to_string();

    println!("{}", table);
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
            &stat.file_id[..8.min(stat.file_id.len())], // Keep short ID for readability
            stat.access_count
        );
    }

    println!();
    Ok(())
}
