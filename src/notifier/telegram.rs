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

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

use crate::health::HealthReport;
use crate::notifier::Notifier;

/// Telegram Bot API notifier
#[derive(Clone)]
pub struct TelegramNotifier {
    pub bot_token: String,
    pub chat_id: String,
    client: Client,
}

impl TelegramNotifier {
    pub fn new(bot_token: String, chat_id: String) -> Self {
        Self {
            bot_token,
            chat_id,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl Notifier for TelegramNotifier {
    async fn notify(&self, report: &HealthReport) -> Result<()> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);

        let mut params = HashMap::new();
        params.insert("chat_id", self.chat_id.clone());
        params.insert("text", report.to_alert_message());
        params.insert("parse_mode", "HTML".to_string());

        let response = self.client.post(&url).json(&params).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Telegram API error: {}", error_text);
        }

        Ok(())
    }

    fn notifier_name(&self) -> String {
        format!("telegram:chat_{}", self.chat_id)
    }
}
