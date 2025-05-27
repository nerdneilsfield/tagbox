use crate::utils::error::Result;
use tagbox_core::config::AppConfig;
use tagbox_core::AuthorManager;

/// Handle author commands
pub async fn handle_author_add(name: &str, config: &AppConfig) -> Result<()> {
    log::info!("Adding author: {}", name);

    let db = tagbox_core::schema::Database::new(&config.database.path).await?;
    let author_manager = AuthorManager::new(db.pool().clone());

    let author = author_manager.create_author(name, &[]).await?;
    println!("Author added with ID: {}", author.id);

    Ok(())
}

/// Handle author removal
pub async fn handle_author_remove(id: &str, config: &AppConfig) -> Result<()> {
    log::info!("Removing author: {}", id);

    let db = tagbox_core::schema::Database::new(&config.database.path).await?;
    let _author_manager = AuthorManager::new(db.pool().clone());

    // Get author details before removal for confirmation
    // let author = author_manager.get_author(id).await?;

    // TODO: Implement delete_author in core
    println!("Author removal not yet implemented in core: {}", id);

    Ok(())
}

/// Handle author merge
pub async fn handle_author_merge(from_id: &str, to_id: &str, config: &AppConfig) -> Result<()> {
    log::info!("Merging author {} into {}", from_id, to_id);

    let db = tagbox_core::schema::Database::new(&config.database.path).await?;
    let author_manager = AuthorManager::new(db.pool().clone());

    // Get author details for confirmation
    let from_author = author_manager.get_author(from_id).await?;
    let to_author = author_manager.get_author(to_id).await?;

    author_manager.merge_authors(from_id, to_id).await?;

    println!(
        "Successfully merged '{}' ({}) into '{}' ({})",
        from_author.name, from_id, to_author.name, to_id
    );

    Ok(())
}

/// List all authors
pub async fn handle_author_list(config: &AppConfig) -> Result<()> {
    log::info!("Listing all authors");

    let db = tagbox_core::schema::Database::new(&config.database.path).await?;
    let _author_manager = AuthorManager::new(db.pool().clone());

    // TODO: Implement list_authors in core
    println!("Author listing not yet implemented in core.");
    println!("Use SQL query directly: SELECT * FROM authors;");

    Ok(())
}
