/*!
 * Copyright (c) 2025 Hangzhou Guanwaii Technology Co., Ltd.
 *
 * This source code is licensed under the MIT License,
 * which is located in the LICENSE file in the source tree's root directory.
 *
 * File: main.rs
 * Author: mingcheng <mingcheng@apache.org>
 * File Created: 2025-11-17 15:34:23
 *
 * Modified By: mingcheng <mingcheng@apache.org>
 * Last Modified: 2025-11-18 15:10:59
 */

/*!
 * ZWatch - ZFS Pool Monitoring and Alerting System
 *
 * A flexible monitoring system for ZFS pools that can fetch status from
 * multiple sources and send notifications through various channels.
 */

mod config;
mod health;
mod notifier;
mod source;

use anyhow::Result;
use std::time::Duration;
use tracing::{error, info, warn};

use config::{Config, DataSourceConfig, NotifierConfig};
use health::HealthChecker;
use notifier::{
    BarkNotifier, ConsoleNotifier, MQNotifier, Notifier, TelegramNotifier, WebhookNotifier,
};
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
use source::LocalCommandDataSource;
use source::{FileDataSource, SSHDataSource, ZpoolDataSource};

struct ZWatch {
    config: Config,
    data_sources: Vec<Box<dyn ZpoolDataSource>>,
    notifiers: Vec<Box<dyn Notifier>>,
}

impl ZWatch {
    fn new(config: Config) -> Result<Self> {
        let data_sources = Self::build_data_sources(&config.sources)?;
        let notifiers = Self::build_notifiers(&config.notifiers)?;

        Ok(Self {
            config,
            data_sources,
            notifiers,
        })
    }

    fn build_data_sources(configs: &[DataSourceConfig]) -> Result<Vec<Box<dyn ZpoolDataSource>>> {
        let mut sources: Vec<Box<dyn ZpoolDataSource>> = Vec::new();

        for config in configs {
            match config {
                DataSourceConfig::File { name: _, path } => {
                    sources.push(Box::new(FileDataSource::new(path.clone())));
                }
                DataSourceConfig::Local {
                    name,
                    command,
                    args,
                } => {
                    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
                    {
                        sources.push(Box::new(LocalCommandDataSource::new(
                            command.clone(),
                            args.clone(),
                        )));
                    }
                    #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
                    {
                        let _ = (command, args); // Silence unused warnings
                        anyhow::bail!(
                            "Local command data source '{}' is only supported on Linux and FreeBSD",
                            name
                        );
                    }
                }
                DataSourceConfig::SSH {
                    name: _,
                    host,
                    user,
                    port,
                    keyfile,
                    command,
                    args,
                } => {
                    let mut ssh_source = SSHDataSource::new(host.clone(), user.clone())
                        .with_command(command.clone(), args.clone());

                    if let Some(port) = port {
                        ssh_source = ssh_source.with_port(*port);
                    }

                    if let Some(keyfile) = keyfile {
                        ssh_source = ssh_source.with_keyfile(keyfile.clone());
                    }

                    sources.push(Box::new(ssh_source));
                }
            }
        }

        Ok(sources)
    }

    fn build_notifiers(configs: &[NotifierConfig]) -> Result<Vec<Box<dyn Notifier>>> {
        let mut notifiers: Vec<Box<dyn Notifier>> = Vec::new();

        for config in configs {
            let notifier: Box<dyn Notifier> = match config {
                NotifierConfig::Console => Box::new(ConsoleNotifier),
                NotifierConfig::Telegram { bot_token, chat_id } => {
                    Box::new(TelegramNotifier::new(bot_token.clone(), chat_id.clone()))
                }
                NotifierConfig::Webhook { url, headers } => {
                    let mut notifier = WebhookNotifier::new(url.clone());
                    for (key, value) in headers {
                        notifier = notifier.with_header(key.clone(), value.clone());
                    }
                    Box::new(notifier)
                }
                NotifierConfig::MQ {
                    amqp_url,
                    exchange,
                    routing_key,
                } => Box::new(MQNotifier::new(
                    amqp_url.clone(),
                    exchange.clone(),
                    routing_key.clone(),
                )),
                NotifierConfig::Bark {
                    server_url,
                    device_key,
                } => Box::new(BarkNotifier::new(server_url.clone(), device_key.clone())),
            };

            notifiers.push(notifier);
        }

        Ok(notifiers)
    }

    async fn check_and_notify(&self) -> Result<()> {
        for source in &self.data_sources {
            let source_name = source.source_name();

            info!("Checking source: {}", source_name);

            match source.fetch_status().await {
                Ok(json_data) => match HealthChecker::check(&json_data) {
                    Ok(reports) => {
                        for report in reports {
                            if !report.is_healthy {
                                warn!(
                                    "Pool '{}' from source '{}' is unhealthy: {}",
                                    report.pool_name, source_name, report.message
                                );

                                self.send_notifications(&report).await;
                            } else {
                                info!(
                                    "Pool '{}' from source '{}' is healthy",
                                    report.pool_name, source_name
                                );

                                if !self.config.settings.notify_on_error_only {
                                    self.send_notifications(&report).await;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to check health for source '{}': {}", source_name, e);
                    }
                },
                Err(e) => {
                    error!(
                        "Failed to fetch status from source '{}': {}",
                        source_name, e
                    );
                }
            }
        }

        Ok(())
    }

    async fn send_notifications(&self, report: &health::HealthReport) {
        for notifier in &self.notifiers {
            let notifier_name = notifier.notifier_name();

            match notifier.notify(report).await {
                Ok(_) => {
                    info!("Successfully sent notification via {}", notifier_name);
                }
                Err(e) => {
                    error!("Failed to send notification via {}: {}", notifier_name, e);
                }
            }
        }
    }

    async fn run(&self) -> Result<()> {
        let interval = Duration::from_secs(self.config.settings.check_interval);

        info!(
            "Starting ZWatch with {} data source(s) and {} notifier(s)",
            self.data_sources.len(),
            self.notifiers.len()
        );
        info!(
            "Check interval: {} seconds",
            self.config.settings.check_interval
        );

        loop {
            if let Err(e) = self.check_and_notify().await {
                error!("Error during check cycle: {}", e);
            }

            info!("Sleeping for {} seconds...", interval.as_secs());
            tokio::time::sleep(interval).await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_line_number(true)
        .init();

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "init" => {
                let config = Config::example();
                let path = "zwatch.toml";
                config.save(path)?;
                println!("Example configuration saved to {}", path);
                return Ok(());
            }
            "check" => {
                // Single check mode
                let config = load_config(args.get(2))?;
                let watcher = ZWatch::new(config)?;
                watcher.check_and_notify().await?;
                return Ok(());
            }
            path if path.ends_with(".toml") => {
                // Run with specific config file
                let config = Config::from_file(path)?;
                let watcher = ZWatch::new(config)?;
                watcher.run().await?;
            }
            _ => {
                print_usage();
                return Ok(());
            }
        }
    } else {
        // Try to load default config
        let config = load_config(None)?;
        let watcher = ZWatch::new(config)?;
        watcher.run().await?;
    }

    Ok(())
}

fn load_config(path: Option<&String>) -> Result<Config> {
    if let Some(path) = path {
        Config::from_file(path)
    } else if std::path::Path::new("zwatch.toml").exists() {
        Config::from_file("zwatch.toml")
    } else {
        info!("No configuration file found, using defaults");
        Ok(Config::default())
    }
}

fn print_usage() {
    println!("ZWatch - ZFS Pool Monitoring System");
    println!();
    println!("Usage:");
    println!("  zwatch                  # Run with default config (zwatch.toml)");
    println!("  zwatch <config.toml>    # Run with specified config file");
    println!("  zwatch init             # Generate example configuration");
    println!("  zwatch check [config]   # Run single check (no loop)");
}
