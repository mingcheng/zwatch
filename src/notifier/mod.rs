/*!
 * Copyright (c) 2025 Hangzhou Guanwaii Technology Co., Ltd.
 *
 * This source code is licensed under the MIT License,
 * which is located in the LICENSE file in the source tree's root directory.
 *
 * File: mod.rs
 * Author: mingcheng <mingcheng@apache.org>
 * File Created: 2025-11-17 15:39:55
 *
 * Modified By: mingcheng <mingcheng@apache.org>
 * Last Modified: 2025-11-18 15:08:21
 */

mod bark;
mod console;
mod telegram;
mod webhook;

use async_trait::async_trait;
pub use bark::BarkNotifier;
pub use console::ConsoleNotifier;
pub use telegram::TelegramNotifier;
pub use webhook::WebhookNotifier;

use crate::health::HealthReport;

/// Trait for sending notifications
#[async_trait]
pub trait Notifier: Send + Sync {
    /// Send a notification with the given message
    async fn notify(&self, report: &HealthReport) -> anyhow::Result<()>;

    /// Get the name of this notifier
    fn name(&self) -> String;
}
