use std::io::{self, Write};
use tagbox_core::{
    config::AppConfig, errors::Result, schema::Database, types::FileUpdateRequest,
    utils::parse_category_string, Editor,
};

/// Handle file edit command
pub async fn handle_edit(
    file_id: &str,
    interactive: bool,
    mv: bool,
    title: Option<String>,
    authors: Option<String>,
    category: Option<String>,
    tags: Option<String>,
    summary: Option<String>,
    year: Option<i32>,
    publisher: Option<String>,
    source: Option<String>,
    config: &AppConfig,
) -> Result<()> {
    let db = Database::new(&config.database.path).await?;
    let editor = Editor::new(db.pool().clone());

    // 获取当前文件信息
    let current_file = editor.get_file_for_edit(file_id).await?;

    println!("Editing file: {}", current_file.title);
    println!("Current path: {}", current_file.path.display());
    println!();

    let update_request = if interactive {
        interactive_edit(&current_file).await?
    } else {
        build_update_request(
            title, authors, category, tags, summary, year, publisher, source,
        )?
    };

    // 预览更改
    let changes = editor.preview_changes(&current_file, &update_request);
    if !changes.is_empty() {
        println!("Proposed changes:");
        for change in &changes {
            println!("  - {}", change);
        }
        println!();

        if interactive || prompt_confirm("Apply these changes?")? {
            // 执行更新
            if let Some(new_path) = editor
                .update_file_with_move(file_id, update_request, mv, config)
                .await?
            {
                println!("✓ File updated and moved to: {}", new_path.display());
            } else {
                println!("✓ File updated successfully");
            }
        } else {
            println!("Changes cancelled");
        }
    } else {
        println!("No changes to apply");
    }

    Ok(())
}

/// Interactive edit mode
async fn interactive_edit(
    current_file: &tagbox_core::types::FileEntry,
) -> Result<FileUpdateRequest> {
    println!("Interactive Edit Mode - Press Enter to keep current value, or type new value:");
    println!();

    let mut update = FileUpdateRequest {
        title: None,
        authors: None,
        year: None,
        publisher: None,
        source: None,
        category1: None,
        category2: None,
        category3: None,
        tags: None,
        summary: None,
        full_text: None,
        is_deleted: None,
        file_metadata: None,
        type_metadata: None,
    };

    // Title
    println!("Title (current: '{}'): ", current_file.title);
    if let Some(new_title) = prompt_optional_string()? {
        update.title = Some(new_title);
    }

    // Authors
    println!("Authors (current: {:?}): ", current_file.authors);
    if let Some(authors_str) = prompt_optional_string()? {
        let authors: Vec<String> = authors_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !authors.is_empty() {
            update.authors = Some(authors);
        }
    }

    // Category
    let current_category = format!(
        "{}{}{}",
        current_file.category1,
        current_file
            .category2
            .as_ref()
            .map_or_else(String::new, |c2| format!("/{}", c2)),
        current_file
            .category3
            .as_ref()
            .map_or_else(String::new, |c3| format!("/{}", c3))
    );
    println!("Category (current: '{}'): ", current_category);
    if let Some(category_str) = prompt_optional_string()? {
        let (cat1, cat2, cat3) = parse_category_string(&category_str)?;
        update.category1 = Some(cat1);
        update.category2 = cat2;
        update.category3 = cat3;
    }

    // Tags
    println!("Tags (current: {:?}): ", current_file.tags);
    if let Some(tags_str) = prompt_optional_string()? {
        let tags: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !tags.is_empty() {
            update.tags = Some(tags);
        }
    }

    // Summary
    println!("Summary (current: {:?}): ", current_file.summary);
    if let Some(new_summary) = prompt_optional_string()? {
        update.summary = Some(new_summary);
    }

    // Year
    println!("Year (current: {:?}): ", current_file.year);
    if let Some(year_str) = prompt_optional_string()? {
        if let Ok(year) = year_str.parse::<i32>() {
            update.year = Some(year);
        }
    }

    // Publisher
    println!("Publisher (current: {:?}): ", current_file.publisher);
    if let Some(new_publisher) = prompt_optional_string()? {
        update.publisher = Some(new_publisher);
    }

    // Source
    println!("Source (current: {:?}): ", current_file.source);
    if let Some(new_source) = prompt_optional_string()? {
        update.source = Some(new_source);
    }

    Ok(update)
}

/// Build update request from command line arguments
fn build_update_request(
    title: Option<String>,
    authors: Option<String>,
    category: Option<String>,
    tags: Option<String>,
    summary: Option<String>,
    year: Option<i32>,
    publisher: Option<String>,
    source: Option<String>,
) -> Result<FileUpdateRequest> {
    let mut update = FileUpdateRequest {
        title,
        authors: None,
        year,
        publisher,
        source,
        category1: None,
        category2: None,
        category3: None,
        tags: None,
        summary,
        full_text: None,
        is_deleted: None,
        file_metadata: None,
        type_metadata: None,
    };

    // Parse authors
    if let Some(authors_str) = authors {
        let authors_vec: Vec<String> = authors_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !authors_vec.is_empty() {
            update.authors = Some(authors_vec);
        }
    }

    // Parse category
    if let Some(category_str) = category {
        let (cat1, cat2, cat3) = parse_category_string(&category_str)?;
        update.category1 = Some(cat1);
        update.category2 = cat2;
        update.category3 = cat3;
    }

    // Parse tags
    if let Some(tags_str) = tags {
        let tags_vec: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !tags_vec.is_empty() {
            update.tags = Some(tags_vec);
        }
    }

    Ok(update)
}

/// Prompt for optional string input
fn prompt_optional_string() -> io::Result<Option<String>> {
    print!("> ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}

/// Prompt for yes/no confirmation
fn prompt_confirm(message: &str) -> io::Result<bool> {
    loop {
        print!("{} (y/N): ", message);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" | "" => return Ok(false),
            _ => println!("Please enter 'y' or 'n'"),
        }
    }
}
