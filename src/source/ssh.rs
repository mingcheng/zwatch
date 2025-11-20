/*!
 * Copyright (c) 2025 Hangzhou Guanwaii Technology Co., Ltd.
 *
 * This source code is licensed under the MIT License,
 * which is located in the LICENSE file in the source tree's root directory.
 *
 * File: ssh.rs
 * Author: mingcheng <mingcheng@apache.org>
 * File Created: 2025-11-17 15:51:11
 *
 * Modified By: mingcheng <mingcheng@apache.org>
 * Last Modified: 2025-11-20 14:49:09
 */

use crate::source::ZpoolDataSource;
use anyhow::Result;
use async_trait::async_trait;
use openssh::{Session, SessionBuilder};
use serde_json::Value;
use tracing::warn;

const DEFAULT_SSH_PORT: u16 = 22;

/// Fetch zpool status via SSH from a remote host
pub struct SSHDataSource {
    pub host: String,
    pub user: String,
    pub port: Option<u16>,
    pub keyfile: Option<String>,
    pub command: String,
    pub args: Vec<String>,
}

impl SSHDataSource {
    pub fn new(host: String, user: String) -> Self {
        Self {
            host,
            user,
            port: None,
            keyfile: None,
            command: "zpool".to_string(),
            args: vec!["status".to_string(), "-j".to_string()],
        }
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = Some(if port == 0 {
            warn!(
                "Invalid SSH port 0 specified, using default port {}",
                DEFAULT_SSH_PORT
            );
            DEFAULT_SSH_PORT
        } else {
            port
        });

        self
    }

    pub fn with_keyfile(mut self, keyfile: String) -> Self {
        self.keyfile = Some(keyfile);
        self
    }

    pub fn with_command(mut self, command: String, args: Vec<String>) -> Self {
        self.command = command;
        self.args = args;
        self
    }

    async fn connect(&self) -> Result<Session> {
        let mut builder = SessionBuilder::default();
        builder.user(self.user.clone());
        builder.port(self.port.unwrap_or(DEFAULT_SSH_PORT));

        if let Some(keyfile) = &self.keyfile {
            builder.keyfile(keyfile);
        }

        let session = builder.connect(&self.host).await?;
        Ok(session)
    }
}

#[async_trait]
impl ZpoolDataSource for SSHDataSource {
    async fn fetch(&self) -> Result<String> {
        let session = self.connect().await.map_err(|e| {
            anyhow::anyhow!(
                "Failed to establish SSH connection to {}@{}: {:?}",
                self.user,
                self.host,
                e
            )
        })?;

        let output = session
            .command(&self.command)
            .args(&self.args)
            .output()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to execute SSH command: {:?}", e))?;

        // Close the session properly
        session.close().await.ok();

        anyhow::ensure!(
            output.status.success(),
            "SSH command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let json_output = String::from_utf8(output.stdout)
            .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in command output: {:?}", e))?;

        // Validate JSON
        serde_json::from_str::<Value>(&json_output)
            .map_err(|e| anyhow::anyhow!("Invalid JSON output: {:?}", e))?;

        Ok(json_output)
    }

    fn name(&self) -> String {
        format!(
            "ssh:{}@{}:{} {}",
            self.user,
            self.host,
            self.command,
            self.args.join(" ")
        )
    }
}

/// Display implementation for SSHDataSource
impl std::fmt::Display for SSHDataSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use crate::source::{SSHDataSource, ZpoolDataSource};

    #[tokio::test]
    async fn test_ssh_data_source() {
        let host = std::env::var("TEST_SSH_HOST").unwrap_or("".to_string());
        let user = std::env::var("TEST_SSH_USER").unwrap_or("".to_string());
        if user.is_empty() || host.is_empty() {
            return;
        }

        let ssh_data_source = SSHDataSource::new(host, user).with_port(22);
        let result = ssh_data_source.fetch().await.unwrap();

        assert!(!result.is_empty());
    }
}
