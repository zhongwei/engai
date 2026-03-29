use engai_core::config::Config;

#[test]
fn test_default_config() {
    let config = Config::default();

    assert_eq!(config.server.port, 3000);
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.ai.provider, "kimi");
    assert_eq!(config.learning.daily_new_words, 20);
    assert_eq!(config.learning.daily_review_limit, 100);
    assert_eq!(config.learning.default_deck, "01_vocab");
    assert_eq!(config.storage.db_path, "~/.engai/engai.db");
    assert_eq!(config.storage.docs_path, "./docs");
}

#[test]
fn test_config_dir() {
    let dir = Config::config_dir();
    let last = dir.file_name().unwrap().to_string_lossy();
    assert_eq!(last, ".engai");
}

#[test]
fn test_config_file_path() {
    let path = Config::config_file_path();
    let last = path.file_name().unwrap().to_string_lossy();
    assert_eq!(last, "config.toml");
    assert!(
        path.to_string_lossy().contains(".engai"),
        "config file path should contain .engai"
    );
}

#[tokio::test]
async fn test_save_and_load_config() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");

    let config = Config::default();
    config.save_to(&path).unwrap();

    let loaded = Config::load_from(path.clone()).unwrap();
    assert_eq!(loaded.server.port, config.server.port);
    assert_eq!(loaded.ai.provider, config.ai.provider);
    assert_eq!(loaded.learning.daily_new_words, config.learning.daily_new_words);
    assert_eq!(loaded.storage.docs_path, config.storage.docs_path);
}

#[test]
fn test_load_missing_config_returns_default() {
    let path = std::path::PathBuf::from("/nonexistent/path/config.toml");
    let config = Config::load_from(path).unwrap();
    assert_eq!(config.server.port, 3000);
    assert_eq!(config.ai.provider, "kimi");
}
