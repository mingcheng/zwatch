/*!
 * Copyright (c) 2025 Hangzhou Guanwaii Technology Co., Ltd.
 *
 * This source code is licensed under the MIT License,
 * which is located in the LICENSE file in the source tree's root directory.
 *
 * File: checker.rs
 * Author: mingcheng <mingcheng@apache.org>
 * File Created: 2025-11-17 15:53:43
 *
 * Modified By: mingcheng <mingcheng@apache.org>
 * Last Modified: 2025-11-18 15:06:46
 */

use anyhow::{Context, Result};

use super::report::{DeviceError, HealthReport};
use super::types::{Pool, Vdev, VdevRoot, ZpoolStatus};

pub struct HealthChecker;

impl HealthChecker {
    pub fn check(json_data: &str) -> Result<Vec<HealthReport>> {
        let status = serde_json::from_str::<ZpoolStatus>(json_data)
            .context("Failed to parse zpool status JSON")?;

        let mut reports = Vec::new();

        for (_, pool) in status.pools {
            let report = Self::check_pool(&pool);
            reports.push(report);
        }

        Ok(reports)
    }

    fn check_pool(pool: &Pool) -> HealthReport {
        let pool_error_count = pool.error_count.parse::<u64>().unwrap_or(0);
        let scan_errors = pool
            .scan_stats
            .as_ref()
            .and_then(|s| s.errors.parse::<u64>().ok())
            .unwrap_or(0);

        let device_errors = Self::collect_device_errors(&pool.vdevs);

        let is_healthy = pool.state.to_uppercase() == "ONLINE"
            && pool_error_count == 0
            && scan_errors == 0
            && device_errors.is_empty();

        let message = if is_healthy {
            "All systems operational".to_string()
        } else {
            let mut msgs = Vec::new();

            if pool.state.to_uppercase() != "ONLINE" {
                msgs.push(format!("Pool state is {}", pool.state));
            }
            if pool_error_count > 0 {
                msgs.push(format!("{} pool errors detected", pool_error_count));
            }
            if scan_errors > 0 {
                msgs.push(format!("{} scan errors detected", scan_errors));
            }
            if !device_errors.is_empty() {
                msgs.push(format!("{} devices have errors", device_errors.len()));
            }

            msgs.join("; ")
        };

        HealthReport {
            pool_name: pool.name.clone(),
            pool_state: pool.state.clone(),
            is_healthy,
            pool_error_count,
            scan_errors,
            device_errors,
            message,
        }
    }

    fn collect_device_errors(vdev_root: &VdevRoot) -> Vec<DeviceError> {
        let mut errors = Vec::new();

        for dev in vdev_root.devices.values() {
            Self::check_vdev(dev, &mut errors);
        }

        errors
    }

    fn check_vdev(vdev: &Vdev, errors: &mut Vec<DeviceError>) {
        let read_errors = vdev.read_errors.parse::<u64>().unwrap_or(0);
        let write_errors = vdev.write_errors.parse::<u64>().unwrap_or(0);
        let checksum_errors = vdev.checksum_errors.parse::<u64>().unwrap_or(0);

        let has_errors = read_errors > 0
            || write_errors > 0
            || checksum_errors > 0
            || vdev.state.to_uppercase() != "ONLINE";

        if has_errors {
            errors.push(DeviceError {
                device_name: vdev.name.clone(),
                device_path: vdev.path.clone(),
                state: vdev.state.clone(),
                read_errors,
                write_errors,
                checksum_errors,
            });
        }

        // Recursively check child vdevs
        if let Some(children) = &vdev.vdevs {
            for device in children.devices.values() {
                Self::check_vdev(device, errors);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healthy_pool() {
        let json = r#"{
            "output_version": {"command": "zpool status", "vers_major": 2, "vers_minor": 0},
            "pools": {
                "tank": {
                    "name": "tank",
                    "state": "ONLINE",
                    "pool_guid": "123456",
                    "txg": "100",
                    "spa_version": "5000",
                    "zpl_version": "5",
                    "error_count": "0",
                    "vdevs": {
                        "root-0": {
                            "name": "root-0",
                            "vdev_type": "root",
                            "guid": "123",
                            "class": "vdev",
                            "state": "ONLINE",
                            "read_errors": "0",
                            "write_errors": "0",
                            "checksum_errors": "0"
                        }
                    }
                }
            }
        }"#;

        let reports = HealthChecker::check(json).unwrap();
        assert_eq!(reports.len(), 1);
        assert!(reports[0].is_healthy);
    }
}
