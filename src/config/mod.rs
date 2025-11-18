/*!
 * Copyright (c) 2025 Hangzhou Guanwaii Technology Co., Ltd.
 *
 * This source code is licensed under the MIT License,
 * which is located in the LICENSE file in the source tree's root directory.
 *
 * File: mod.rs
 * Author: mingcheng <mingcheng@apache.org>
 * File Created: 2025-11-17 15:55:18
 *
 * Modified By: mingcheng <mingcheng@apache.org>
 * Last Modified: 2025-11-17 17:43:18
 */

mod types;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
pub use types::Config;

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    /// Check interval in seconds
    #[serde(default = "default_check_interval")]
    pub check_interval: u64,

    /// Only notify on errors (don't notify when healthy)
    #[serde(default = "default_notify_on_error_only")]
    pub notify_on_error_only: bool,

    /// Enable verbose logging
    #[serde(default)]
    pub verbose: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum NotifierConfig {
    Telegram {
        bot_token: String,
        chat_id: String,
    },
    Webhook {
        url: String,
        #[serde(default)]
        headers: HashMap<String, String>,
    },
    Bark {
        server_url: String,
        device_key: String,
    },
    Console,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum DataSourceConfig {
    File {
        name: String,
        path: PathBuf,
    },
    Local {
        name: String,
        #[serde(default = "default_command")]
        command: String,
        #[serde(default = "default_args")]
        args: Vec<String>,
    },
    SSH {
        name: String,
        host: String,
        user: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        port: Option<u16>,
        #[serde(skip_serializing_if = "Option::is_none")]
        keyfile: Option<String>,
        #[serde(default = "default_command")]
        command: String,
        #[serde(default = "default_args")]
        args: Vec<String>,
    },
}

fn default_command() -> String {
    "zpool".to_string()
}

fn default_args() -> Vec<String> {
    vec!["status".to_string(), "-j".to_string()]
}

fn default_check_interval() -> u64 {
    300 // 5 minutes
}

fn default_notify_on_error_only() -> bool {
    true
}
