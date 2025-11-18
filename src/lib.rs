/*!
 * Copyright (c) 2025 Hangzhou Guanwaii Technology Co., Ltd.
 *
 * This source code is licensed under the MIT License,
 * which is located in the LICENSE file in the source tree's root directory.
 *
 * File: lib.rs
 * Author: mingcheng <mingcheng@apache.org>
 * File Created: 2025-11-17 15:43:57
 *
 * Modified By: mingcheng <mingcheng@apache.org>
 * Last Modified: 2025-11-18 15:10:52
 */

pub mod config;
pub mod health;
pub mod notifier;
pub mod source;

pub use config::Config;
pub use health::{HealthChecker, HealthReport};
