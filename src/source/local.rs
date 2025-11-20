/*!
 * Copyright (c) 2025 Hangzhou Guanwaii Technology Co., Ltd.
 *
 * This source code is licensed under the MIT License,
 * which is located in the LICENSE file in the source tree's root directory.
 *
 * File: local.rs
 * Author: mingcheng <mingcheng@apache.org>
 * File Created: 2025-11-17 15:51:09
 *
 * Modified By: mingcheng <mingcheng@apache.org>
 * Last Modified: 2025-11-19 10:19:54
 */

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use tokio::process::Command;

use crate::source::ZpoolDataSource;

/// Fetch zpool status by executing local command
pub struct LocalCommandDataSource {
    pub command: String,
    pub args: Vec<String>,
}

impl LocalCommandDataSource {
    pub fn new(command: String, args: Vec<String>) -> Self {
        Self { command, args }
    }

    /// Create a default instance that runs "zpool status -j"
    #[allow(dead_code)]
    pub fn default_zpool() -> Self {
        Self {
            command: "zpool".to_string(),
            args: vec!["status".to_string(), "-j".to_string()],
        }
    }
}

#[async_trait]
impl ZpoolDataSource for LocalCommandDataSource {
    async fn fetch(&self) -> Result<String> {
        let output = Command::new(&self.command)
            .args(&self.args)
            .output()
            .await?;

        anyhow::ensure!(
            output.status.success(),
            "Command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let json_output = String::from_utf8(output.stdout)?;

        // Validate JSON
        serde_json::from_str::<Value>(&json_output)?;

        Ok(json_output)
    }

    fn name(&self) -> String {
        format!("local:{} {}", self.command, self.args.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_invalid_command() {
        let source = LocalCommandDataSource::new("nonexistent_command_xyz".to_string(), vec![]);
        assert!(source.fetch().await.is_err());
    }
}
