/*!
 * Copyright (c) 2025 Hangzhou Guanwaii Technology Co., Ltd.
 *
 * This source code is licensed under the MIT License,
 * which is located in the LICENSE file in the source tree's root directory.
 *
 * File: bark.rs
 * Author: mingcheng <mingcheng@apache.org>
 * File Created: 2025-11-17 15:52:13
 *
 * Modified By: mingcheng <mingcheng@apache.org>
 * Last Modified: 2025-11-18 12:23:47
 */

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;

use crate::health::HealthReport;
use crate::notifier::Notifier;

/// Bark iOS notification service
#[derive(Clone)]
pub struct BarkNotifier {
    pub server_url: String,
    pub device_key: String,
    client: Client,
}

impl BarkNotifier {
    pub fn new(server_url: String, device_key: String) -> Self {
        Self {
            server_url,
            device_key,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl Notifier for BarkNotifier {
    async fn notify(&self, report: &HealthReport) -> Result<()> {
        let title = format!("ZFS Alert: {}", report.pool_name);
        let url = format!(
            "{}/{}/{}",
            self.server_url.trim_end_matches('/'),
            self.device_key,
            urlencoding::encode(&title)
        );

        let message = report.to_alert_message();
        let body = urlencoding::encode(&message);

        let full_url = format!("{}?body={}", url, body);

        let response = self.client.get(&full_url).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Bark error: {}", error_text);
        }

        Ok(())
    }

    fn notifier_name(&self) -> String {
        format!("bark:{}", self.device_key)
    }
}
