use std::fs;
use tagbox_core::config::AppConfig;
use tempfile::tempdir;

#[tokio::test]
async fn test_load_and_validate_config() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");
    let toml = r#"
        [import.paths]
        storage_dir = "./data"
        rename_template = "{title}_{authors}"
        classify_template = "{category1}/{filename}"

        [import.metadata]
        prefer_json = true
        fallback_pdf = true
        default_category = "misc"

        [search]
        default_limit = 10
        enable_fts = true
        fts_language = "simple"

        [database]
        path = "./db.sqlite"
        journal_mode = "WAL"
        sync_mode = "NORMAL"

        [hash]
        algorithm = "blake2b"
    "#;
    fs::write(&config_path, toml).unwrap();

    let cfg = AppConfig::from_file(&config_path).await.unwrap();
    assert_eq!(cfg.search.default_limit, 10);
    cfg.validate().unwrap();
}
