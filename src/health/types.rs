/*!
 * Copyright (c) 2025 Hangzhou Guanwaii Technology Co., Ltd.
 *
 * This source code is licensed under the MIT License,
 * which is located in the LICENSE file in the source tree's root directory.
 *
 * File: types.rs
 * Author: mingcheng <mingcheng@apache.org>
 * File Created: 2025-11-17 15:53:39
 *
 * Modified By: mingcheng <mingcheng@apache.org>
 * Last Modified: 2025-11-18 15:07:09
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ZpoolStatus {
    pub output_version: OutputVersion,
    pub pools: HashMap<String, Pool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputVersion {
    pub command: String,
    pub vers_major: u32,
    pub vers_minor: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pool {
    pub name: String,
    pub state: String,
    pub pool_guid: String,
    pub txg: String,
    pub spa_version: String,
    pub zpl_version: String,
    pub scan_stats: Option<ScanStats>,
    pub vdevs: VdevRoot,
    pub error_count: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanStats {
    pub function: String,
    pub state: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub to_examine: Option<String>,
    pub examined: Option<String>,
    pub skipped: Option<String>,
    pub processed: Option<String>,
    pub errors: String,
    pub bytes_per_scan: Option<String>,
    pub pass_start: Option<String>,
    pub scrub_pause: Option<String>,
    pub scrub_spent_paused: Option<String>,
    pub issued_bytes_per_scan: Option<String>,
    pub issued: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VdevRoot {
    #[serde(flatten)]
    pub devices: HashMap<String, Vdev>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vdev {
    pub name: String,
    pub vdev_type: String,
    pub guid: String,
    pub class: String,
    pub state: String,
    pub alloc_space: Option<String>,
    pub total_space: Option<String>,
    pub def_space: Option<String>,
    pub rep_dev_size: Option<String>,
    pub phys_space: Option<String>,
    pub read_errors: String,
    pub write_errors: String,
    pub checksum_errors: String,
    pub slow_ios: Option<String>,
    pub path: Option<String>,
    pub phys_path: Option<String>,
    pub devid: Option<String>,
    pub vdevs: Option<VdevRoot>,
}
