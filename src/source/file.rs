/*!
 * Copyright (c) 2025 Hangzhou Guanwaii Technology Co., Ltd.
 *
 * This source code is licensed under the MIT License,
 * which is located in the LICENSE file in the source tree's root directory.
 *
 * File: file.rs
 * Author: mingcheng <mingcheng@apache.org>
 * File Created: 2025-11-17 15:51:07
 *
 * Modified By: mingcheng <mingcheng@apache.org>
 * Last Modified: 2025-11-17 18:27:02
 */

use crate::source::ZpoolDataSource;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::path::PathBuf;
use tokio::fs;

/// Fetch zpool status from a local JSON file
pub struct FileDataSource {
    pub file_path: PathBuf,
}

impl FileDataSource {
    pub fn new(file_path: PathBuf) -> Self {
        Self { file_path }
    }
}

#[async_trait]
impl ZpoolDataSource for FileDataSource {
    async fn fetch(&self) -> Result<String> {
        let content = fs::read_to_string(&self.file_path).await?;

        // Validate it's valid JSON
        serde_json::from_str::<Value>(&content)?;

        Ok(content)
    }

    fn name(&self) -> String {
        format!("file:{}", self.file_path.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_file_datasource_invalid_path() {
        if let Ok(path) = env::var("TEST_FILE_PATH") {
            let source = FileDataSource::new(PathBuf::from(path));
            let status = source.fetch().await.unwrap();
            assert!(!status.is_empty());
        }
    }
}
