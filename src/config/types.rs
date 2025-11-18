/*!
 * Copyright (c) 2025 Hangzhou Guanwaii Technology Co., Ltd.
 *
 * This source code is licensed under the MIT License,
 * which is located in the LICENSE file in the source tree's root directory.
 *
 * File: types.rs
 * Author: mingcheng <mingcheng@apache.org>
 * File Created: 2025-11-17 15:55:16
 *
 * Modified By: mingcheng <mingcheng@apache.org>
 * Last Modified: 2025-11-17 17:43:39
 */

use crate::config::{DataSourceConfig, NotifierConfig, Settings};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Config {
    /// Data sources to monitor
    #[serde(rename = "sources")]
    pub sources: Vec<DataSourceConfig>,

    /// Notification targets
    #[serde(rename = "notifiers")]
    pub notifiers: Vec<NotifierConfig>,

    /// Global settings
    #[serde(rename = "settings")]
    pub settings: Settings,
}

/// Default command for local data source
impl Default for Config {
    fn default() -> Self {
        Self {
            sources: vec![DataSourceConfig::Local {
                name: "localhost".to_string(),
                command: "zpool".to_string(),
                args: vec!["status".to_string(), "-j".to_string()],
            }],
            notifiers: vec![NotifierConfig::Console],
            settings: Settings {
                check_interval: 300,
                notify_on_error_only: true,
                verbose: false,
            },
        }
    }
}

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load configuration from environment variables and files
    #[allow(dead_code)]
    pub fn load() -> anyhow::Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("zwatch").required(false))
            .add_source(config::Environment::with_prefix("ZWATCH"))
            .build()?;

        let config: Config = config.try_deserialize()?;
        Ok(config)
    }

    /// Save configuration to a TOML file
    pub fn save(&self, path: &str) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Generate an example configuration
    pub fn example() -> Self {
        Self {
            sources: vec![
                DataSourceConfig::Local {
                    name: "localhost".to_string(),
                    command: "zpool".to_string(),
                    args: vec!["status".to_string(), "-j".to_string()],
                },
                DataSourceConfig::SSH {
                    name: "remote-server".to_string(),
                    host: "192.168.1.100".to_string(),
                    user: "root".to_string(),
                    port: Some(22),
                    keyfile: Some("/home/user/.ssh/id_rsa".to_string()),
                    command: "zpool".to_string(),
                    args: vec!["status".to_string(), "-j".to_string()],
                },
                DataSourceConfig::File {
                    name: "test-file".to_string(),
                    path: PathBuf::from("/path/to/zpool_status.json"),
                },
            ],
            notifiers: vec![
                NotifierConfig::Console,
                NotifierConfig::Telegram {
                    bot_token: "YOUR_BOT_TOKEN".to_string(),
                    chat_id: "YOUR_CHAT_ID".to_string(),
                },
                NotifierConfig::Webhook {
                    url: "https://example.com/webhook".to_string(),
                    headers: {
                        let mut h = HashMap::new();
                        h.insert("Authorization".to_string(), "Bearer TOKEN".to_string());
                        h
                    },
                },
                NotifierConfig::Bark {
                    server_url: "https://api.day.app".to_string(),
                    device_key: "YOUR_DEVICE_KEY".to_string(),
                },
            ],
            settings: Settings {
                check_interval: 300,
                notify_on_error_only: true,
                verbose: false,
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let config = Config::example();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        // println!("Serialized Config:\n{}", toml_str);

        let deserialized_config: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.sources.len(), deserialized_config.sources.len());
        assert_eq!(config.notifiers.len(), deserialized_config.notifiers.len());
        assert_eq!(
            config.settings.check_interval,
            deserialized_config.settings.check_interval
        );
    }

    #[test]
    fn test_config_default() {
        let default_config = Config::default();
        assert_eq!(default_config.sources.len(), 1);
        assert_eq!(default_config.notifiers.len(), 1);
        assert_eq!(default_config.settings.check_interval, 300);
        assert!(default_config.settings.notify_on_error_only);
        assert!(!default_config.settings.verbose);
    }

    #[test]
    fn test_config_from_file() {
        let config = Config::example();
        let path = "test_config.toml";
        config.save(path).unwrap();

        let loaded_config = Config::from_file(path).unwrap();
        assert_eq!(config.sources.len(), loaded_config.sources.len());
        assert_eq!(config.notifiers.len(), loaded_config.notifiers.len());
        std::fs::remove_file(path).unwrap();
    }
}
