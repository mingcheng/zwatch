/*!
 * Copyright (c) 2025 Hangzhou Guanwaii Technology Co., Ltd.
 *
 * This source code is licensed under the MIT License,
 * which is located in the LICENSE file in the source tree's root directory.
 *
 * File: mod.rs
 * Author: mingcheng <mingcheng@apache.org>
 * File Created: 2025-11-17 15:38:34
 *
 * Modified By: mingcheng <mingcheng@apache.org>
 * Last Modified: 2025-11-17 18:26:16
 */

mod file;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod local;
mod ssh;

use async_trait::async_trait;
pub use file::FileDataSource;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub use local::LocalCommandDataSource;

pub use ssh::SSHDataSource;

/// Trait for fetching ZFS pool status in JSON format
#[async_trait]
pub trait ZpoolDataSource: Send + Sync {
    /// Fetch the zpool status as a JSON string
    async fn fetch(&self) -> anyhow::Result<String>;

    /// Get a descriptive name for this data source
    fn name(&self) -> String;
}
