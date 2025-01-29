use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub storage: StorageConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct StorageConfig {
    pub db_path: String,
    pub page_size: u64,
    pub cache_size: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LoggingConfig {
    pub level: String,
    pub file: String,
    pub max_size_mb: u64,
    pub rotate: bool,
    pub max_files: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage: StorageConfig {
                db_path: "./ferrodb/database.fdb".to_string(),
                page_size: 4096,
                cache_size: 10,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file: "./ferrodb/log.log".to_string(),
                max_size_mb: 100,
                rotate: true,
                max_files: 5,
            },
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid YAML: {0}")]
    InvalidYaml(String),
}

impl Config {
    /// Initialize the configuration, optionally from a YAML file
    pub fn new<P: AsRef<Path>>(config_path: Option<P>) -> Result<Self, ConfigError> {
        let mut config = Config::default();

        // If config path is provided, override defaults with file values
        if let Some(path) = config_path {
            let config_str = fs::read_to_string(&path).map_err(|e| {
                ConfigError::FileNotFound(format!(
                    "Could not read config file {}: {}",
                    path.as_ref().display(),
                    e
                ))
            })?;

            config = serde_yaml::from_str(&config_str).map_err(|e| {
                ConfigError::InvalidYaml(format!("Invalid YAML in config file: {}", e))
            })?;
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default() {
        let config = Config::new(None::<&str>).unwrap();

        assert_eq!(config.storage.db_path, "./ferrodb/database.fdb");
        assert_eq!(config.storage.page_size, 4096);
        assert_eq!(config.storage.cache_size, 10);
    }

    #[test]
    fn test_load_config() {
        let config_content = r#"
            storage:
                db_path: "/var/lib/ferrodb/data.fdb"
                page_size: 8192
                cache_size: 20
            logging:
                level: "debug"
                file: "/var/log/ferrodb/db.log"
                max_size_mb: 200
                rotate: true
                max_files: 10
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, config_content).unwrap();

        let config = Config::new(Some(temp_file.path())).unwrap();

        assert_eq!(config.storage.db_path, "/var/lib/ferrodb/data.fdb");
        assert_eq!(config.storage.page_size, 8192);
        assert_eq!(config.storage.cache_size, 20);
        assert_eq!(config.logging.level, "debug");
        assert_eq!(config.logging.file, "/var/log/ferrodb/db.log");
        assert_eq!(config.logging.max_size_mb, 200);
        assert_eq!(config.logging.rotate, true);
        assert_eq!(config.logging.max_files, 10);
    }

    #[test]
    fn test_invalid_yaml() {
        let invalid_content = "invalid: yaml: : content";
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, invalid_content).unwrap();

        let result = Config::new(Some(temp_file.path()));
        assert!(matches!(result, Err(ConfigError::InvalidYaml(_))));
    }

    #[test]
    fn test_unknown_keys() {
        let config_content = r#"
            storage:
                db_path: "/var/lib/ferrodb/data.fdb"
                page_size: 8192
                unknown_key: "should fail"
            logging:
                level: "debug"
                file: "/var/log/ferrodb/db.log"
                max_size_mb: 200
                rotate: true
                max_files: 10
            extra_section:
                should: "fail"
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, config_content).unwrap();

        let result = Config::new(Some(temp_file.path()));
        assert!(matches!(result, Err(ConfigError::InvalidYaml(_))));
    }
}
