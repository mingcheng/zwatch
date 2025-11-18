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

use anyhow::Result;
use async_trait::async_trait;

use crate::health::HealthReport;
use crate::notifier::Notifier;

/// Console/stdout notifier for debugging
pub struct ConsoleNotifier;

#[async_trait]
impl Notifier for ConsoleNotifier {
    async fn notify(&self, report: &HealthReport) -> Result<()> {
        println!("\n{}", "=".repeat(60));
        println!("NOTIFICATION");
        println!("{}", "=".repeat(60));
        println!("{}", report.to_alert_message());
        println!("{}", "=".repeat(60));
        Ok(())
    }

    fn notifier_name(&self) -> String {
        "console".to_string()
    }
}
