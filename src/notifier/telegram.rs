/*!
 * Copyright (c) 2025 Hangzhou Guanwaii Technology Co., Ltd.
 *
 * This source code is licensed under the MIT License,
 * which is located in the LICENSE file in the source tree's root directory.
 *
 * File: telegram.rs
 * Author: mingcheng <mingcheng@apache.org>
 * File Created: 2025-11-17 15:52:07
 *
 * Modified By: mingcheng <mingcheng@apache.org>
 * Last Modified: 2025-11-18 12:24:03
 */

use crate::health::HealthReport;
use crate::notifier::Notifier;
use anyhow::Result;
use async_trait::async_trait;
use reqwest::{ClientBuilder, Proxy};
use std::env;

use teloxide::{Bot, prelude::Requester};
use tracing::trace;

/// Telegram Bot API notifier
#[derive(Clone)]
pub struct TelegramNotifier {
    bot: Bot,
    pub chat_id: String,
}

impl TelegramNotifier {
    pub fn new(bot_token: String, chat_id: String) -> Self {
        let client = env::var("ALL_PROXY")
            .ok()
            .and_then(|proxy| {
                trace!("Using HTTP proxy for Telegram notifier: {}", proxy);
                ClientBuilder::new()
                    .timeout(std::time::Duration::from_secs(30))
                    .proxy(Proxy::all(proxy).ok()?)
                    .build()
                    .ok()
            })
            .unwrap_or_else(|| {
                ClientBuilder::new()
                    .timeout(std::time::Duration::from_secs(30))
                    .build()
                    .unwrap_or_default()
            });

        let bot = Bot::with_client(bot_token, client);

        Self { bot, chat_id }
    }
}

#[async_trait]
impl Notifier for TelegramNotifier {
    async fn notify(&self, report: &HealthReport) -> Result<()> {
        self.bot
            .send_message(self.chat_id.clone(), report.to_alert_message())
            .await?;

        Ok(())
    }

    fn name(&self) -> String {
        format!("telegram:chat_{}", self.chat_id)
    }
}

#[cfg(test)]
mod tests {
    use crate::health::{DeviceError, HealthReport};
    use crate::notifier::Notifier;

    #[tokio::test]
    async fn test_telegram_notifier() {
        let bot_token = std::env::var("TELEGRAM_BOT_TOKEN").unwrap_or("".to_string());
        let chat_id = std::env::var("TELEGRAM_CHAT_ID").unwrap_or("".to_string());
        if bot_token.is_empty() || chat_id.is_empty() {
            return;
        }

        let notifier = super::TelegramNotifier::new(bot_token.to_string(), chat_id.to_string());

        notifier
            .notify(&HealthReport {
                pool_name: "tank".to_string(),
                pool_state: "DEGRADED".to_string(),
                is_healthy: false,
                pool_error_count: 5,
                scan_errors: 1,
                device_errors: vec![DeviceError {
                    device_name: "sda".to_string(),
                    device_path: Some("/dev/sda".to_string()),
                    state: "DEGRADED".to_string(),
                    read_errors: 10,
                    write_errors: 5,
                    checksum_errors: 2,
                }],
                message: "Test alert message".to_string(),
            })
            .await
            .unwrap();
    }
}
