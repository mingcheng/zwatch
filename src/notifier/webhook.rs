/*!
 * Copyright (c) 2025 Hangzhou Guanwaii Technology Co., Ltd.
 *
 * This source code is licensed under the MIT License,
 * which is located in the LICENSE file in the source tree's root directory.
 *
 * File: webhook.rs
 * Author: mingcheng <mingcheng@apache.org>
 * File Created: 2025-11-17 15:52:09
 *
 * Modified By: mingcheng <mingcheng@apache.org>
 * Last Modified: 2025-11-18 12:24:27
 */

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

use crate::health::HealthReport;
use crate::notifier::Notifier;

/// Generic webhook notifier (POST JSON)
#[derive(Clone)]
pub struct WebhookNotifier {
    pub url: String,
    pub headers: HashMap<String, String>,
    client: Client,
}

impl WebhookNotifier {
    pub fn new(url: String) -> Self {
        Self {
            url,
            headers: HashMap::new(),
            client: Client::new(),
        }
    }

    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    #[allow(dead_code)]
    pub fn with_auth_token(self, token: String) -> Self {
        self.with_header("Authorization".to_string(), format!("Bearer {}", token))
    }
}

#[async_trait]
impl Notifier for WebhookNotifier {
    async fn notify(&self, report: &HealthReport) -> Result<()> {
        let mut request = self.client.post(&self.url);

        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        let payload = serde_json::json!({
            "pool_name": report.pool_name,
            "pool_state": report.pool_state,
            "is_healthy": report.is_healthy,
            "message": report.to_alert_message(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "details": report
        });

        let response = request.json(&payload).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Webhook error: {}", error_text);
        }

        Ok(())
    }

    fn name(&self) -> String {
        format!("webhook:{}", self.url)
    }
}
