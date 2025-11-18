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
 * Last Modified: 2025-11-18 23:17:12
 */

use crate::source::ZpoolDataSource;
use anyhow::Result;
use async_trait::async_trait;
use openssh::{Session, SessionBuilder};
use serde_json::Value;

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
        self.port = Some(port);
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

        if let Some(port) = self.port {
            builder.port(port);
        }

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
        let session = self.connect().await?;

        let output = session
            .command(&self.command)
            .args(&self.args)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("SSH command failed: {}", stderr);
        }

        let json_output = String::from_utf8(output.stdout)?;

        // Validate JSON
        serde_json::from_str::<Value>(&json_output)?;

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
