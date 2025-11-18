/*!
 * Copyright (c) 2025 Hangzhou Guanwaii Technology Co., Ltd.
 *
 * This source code is licensed under the MIT License,
 * which is located in the LICENSE file in the source tree's root directory.
 *
 * File: console.rs
 * Author: mingcheng <mingcheng@apache.org>
 * File Created: 2025-11-17 15:52:15
 *
 * Modified By: mingcheng <mingcheng@apache.org>
 * Last Modified: 2025-11-18 12:23:54
 */

use crate::health::HealthReport;
use crate::notifier::Notifier;
use anyhow::Result;
use async_trait::async_trait;
use tracing::info;

/// Console/stdout notifier for debugging
pub struct ConsoleNotifier;

#[async_trait]
impl Notifier for ConsoleNotifier {
    async fn notify(&self, report: &HealthReport) -> Result<()> {
        info!("\n{}", "=".repeat(60));
        info!("NOTIFICATION");
        info!("{}", "=".repeat(60));
        info!("{}", report.to_alert_message());
        info!("{}", "=".repeat(60));
        Ok(())
    }

    fn name(&self) -> String {
        "console".to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::health::HealthReport;
    use crate::notifier::Notifier;

    #[tokio::test]
    async fn test_console_notifier() {
        let notifier = Box::new(super::ConsoleNotifier);

        notifier
            .notify(&HealthReport {
                pool_name: "testpool".to_string(),
                pool_state: "".to_string(),
                is_healthy: true,
                pool_error_count: 0,
                scan_errors: 0,
                device_errors: vec![],
                message: "".to_string(),
            })
            .await
            .unwrap();
    }
}
