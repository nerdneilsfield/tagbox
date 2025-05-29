use std::sync::{Arc, Mutex};
use tagbox_core::{config::AppConfig, errors::Result, schema::Database, Editor};

/// Handle rebuild command
pub async fn handle_rebuild(
    file_id: Option<String>,
    apply: bool,
    workers: usize,
    config: &AppConfig,
) -> Result<()> {
    let db = Database::new(&config.database.path).await?;
    let editor = Editor::new(db.pool().clone());

    if let Some(id) = file_id {
        // 重建单个文件
        handle_single_file_rebuild(&editor, &id, apply, config).await
    } else {
        // 重建所有文件
        handle_all_files_rebuild(&editor, apply, workers, config).await
    }
}

/// Handle single file rebuild
async fn handle_single_file_rebuild(
    editor: &Editor,
    file_id: &str,
    apply: bool,
    config: &AppConfig,
) -> Result<()> {
    println!("Checking file: {}", file_id);

    match editor.check_file_path(file_id, config).await? {
        Some(expected_path) => {
            let current_file = editor.get_file(file_id).await?;
            let current_absolute = config.import.paths.storage_dir.join(&current_file.path);

            println!("File needs to be moved:");
            println!("  From: {}", current_absolute.display());
            println!("  To:   {}", expected_path.display());

            if apply {
                match editor.rebuild_file_path(file_id, config).await? {
                    Some(new_path) => {
                        println!("✓ File moved to: {}", new_path.display());
                    }
                    None => {
                        println!("ℹ File was already in the correct location");
                    }
                }
            } else {
                println!("ℹ Dry run mode - use --apply to actually move the file");
            }
        }
        None => {
            println!("✓ File is already in the correct location");
        }
    }

    Ok(())
}

/// Handle all files rebuild
async fn handle_all_files_rebuild(
    editor: &Editor,
    apply: bool,
    workers: usize,
    config: &AppConfig,
) -> Result<()> {
    println!("Scanning all files for path mismatches...");

    if apply {
        println!("⚠️  APPLY MODE - Files will be actually moved!");
    } else {
        println!("🔍 DRY RUN MODE - No files will be moved (use --apply to move files)");
    }

    println!("Using {} parallel workers", workers);
    println!();

    // Progress tracking
    let completed = Arc::new(Mutex::new(0usize));
    let total = Arc::new(Mutex::new(0usize));

    let completed_clone = completed.clone();
    let total_clone = total.clone();

    let progress_callback = Box::new(move |current: usize, total_files: usize| {
        {
            let mut completed_guard = completed_clone.lock().unwrap();
            *completed_guard = current;
        }
        {
            let mut total_guard = total_clone.lock().unwrap();
            *total_guard = total_files;
        }

        if current % 10 == 0 || current == total_files {
            println!("Progress: {}/{} files processed", current, total_files);
        }
    });

    let results = editor
        .rebuild_all_files(config, !apply, Some(progress_callback))
        .await?;

    let total_checked = *total.lock().unwrap();
    let moves_needed = results.len();

    println!();
    println!("Rebuild summary:");
    println!("  Total files checked: {}", total_checked);
    println!("  Files needing relocation: {}", moves_needed);

    if !results.is_empty() {
        println!();
        if apply {
            println!("Files moved:");
        } else {
            println!("Files that would be moved:");
        }

        for (file_id, old_path, new_path) in &results {
            println!("  [{}]", file_id);
            println!("    From: {}", old_path.display());
            println!("    To:   {}", new_path.display());
            println!();
        }

        if !apply {
            println!("Use --apply to actually move these files");
        }
    } else {
        println!("✓ All files are in their correct locations");
    }

    Ok(())
}
