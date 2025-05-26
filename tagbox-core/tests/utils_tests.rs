use tagbox_core::utils::*;
use tempfile::tempdir;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

#[tokio::test]
async fn test_calculate_file_hash() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("sample.txt");
    let mut file = File::create(&file_path).unwrap();
    write!(file, "hello world").unwrap();

    let hash = calculate_file_hash(&file_path).await.unwrap();
    assert_eq!(hash.len(), 64); // SHA-256 hex length
}

#[test]
fn test_normalize_and_ensure_dir() {
    let dir = tempdir().unwrap();
    let nested = dir.path().join("a/b");
    ensure_dir_exists(&nested).unwrap();
    assert!(nested.exists());

    let normalized = normalize_path(&nested).unwrap();
    assert!(normalized.is_absolute());
}

#[test]
fn test_uuid_and_time_helpers() {
    let uuid1 = generate_uuid();
    let uuid2 = generate_uuid();
    assert_ne!(uuid1, uuid2);

    let now = current_time();
    let formatted = format_datetime_for_db(&now);
    let parsed = parse_datetime_from_db(&formatted).unwrap();
    assert_eq!(parsed, now);
}

#[tokio::test]
async fn test_safe_copy_file() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let dest = dir.path().join("dest.txt");
    fs::write(&src, b"data").unwrap();

    safe_copy_file(&src, &dest).await.unwrap();
    assert!(dest.exists());
    let content = fs::read(&dest).unwrap();
    assert_eq!(content, b"data");
}
