/*!
 * Copyright (c) 2025 Hangzhou Guanwaii Technology Co., Ltd.
 *
 * This source code is licensed under the MIT License,
 * which is located in the LICENSE file in the source tree's root directory.
 *
 * File: report.rs
 * Author: mingcheng <mingcheng@apache.org>
 * File Created: 2025-11-17 15:53:41
 *
 * Modified By: mingcheng <mingcheng@apache.org>
 * Last Modified: 2025-11-19 23:06:03
 */

use std::fmt;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DeviceError {
    pub device_name: String,
    pub device_path: Option<String>,
    pub state: String,
    pub read_errors: u64,
    pub write_errors: u64,
    pub checksum_errors: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthReport {
    pub pool_name: String,
    pub pool_state: String,
    pub is_healthy: bool,
    pub pool_error_count: u64,
    pub scan_errors: u64,
    pub device_errors: Vec<DeviceError>,
    pub message: String,
}

impl fmt::Display for HealthReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_alert_message())
    }
}

impl HealthReport {
    pub fn to_alert_message(&self) -> String {
        if self.is_healthy {
            format!("✅ ZFS Pool '{}' is ONLINE and healthy", self.pool_name)
        } else {
            let mut msg = format!("⚠️ ZFS Pool '{}' has issues!\n", self.pool_name);
            msg.push_str(&format!("State: {}\n", self.pool_state));
            msg.push_str(&format!("Pool Errors: {}\n", self.pool_error_count));
            msg.push_str(&format!("Scan Errors: {}\n", self.scan_errors));

            if !self.device_errors.is_empty() {
                msg.push_str("\nDevice Errors:\n");
                for dev in &self.device_errors {
                    msg.push_str(&format!(
                        "- {} [{}]: R:{} W:{} C:{}\n",
                        dev.device_name,
                        dev.state,
                        dev.read_errors,
                        dev.write_errors,
                        dev.checksum_errors
                    ));
                }
            }

            msg.push_str(&format!("\nDetails: {}", self.message));
            msg
        }
    }
}
