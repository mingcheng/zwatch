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
 * Last Modified: 2025-11-19 19:19:17
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
use clap::{Parser, Subcommand};
use std::time::Duration;
use tracing::{error, info, warn};

use config::{Config, DataSourceConfig, NotifierConfig};
use health::HealthChecker;
use notifier::{BarkNotifier, ConsoleNotifier, Notifier, TelegramNotifier, WebhookNotifier};
use source::LocalCommandDataSource;
use source::{FileDataSource, SSHDataSource, ZpoolDataSource};

/// ZFS Pool Monitoring and Alerting System
#[derive(Parser)]
#[command(name = "zwatch")]
#[command(version, about, long_about = None)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate example configuration file
    Init {
        /// Output path for the configuration file
        #[arg(short, long, default_value = "zwatch.toml")]
        output: String,
    },
    /// Run a single check without continuous monitoring
    Check {
        /// Configuration file path
        #[arg(short, long)]
        config: Option<String>,
    },
}

struct ZWatch {
    config: Config,
    sources: Vec<Box<dyn ZpoolDataSource>>,
    notifiers: Vec<Box<dyn Notifier>>,
}

impl ZWatch {
    fn new(config: Config) -> Result<Self> {
        let sources = Self::build_sources(&config.sources)?;
        let notifiers = Self::build_notifiers(&config.notifiers)?;

        Ok(Self {
            config,
            sources,
            notifiers,
        })
    }

    fn build_sources(configs: &[DataSourceConfig]) -> Result<Vec<Box<dyn ZpoolDataSource>>> {
        let mut sources: Vec<Box<dyn ZpoolDataSource>> = Vec::new();

        for config in configs {
            match config {
                DataSourceConfig::File { name: _, path } => {
                    sources.push(Box::new(FileDataSource::new(path.clone())));
                }
                DataSourceConfig::Local {
                    name: _,
                    command,
                    args,
                } => {
                    sources.push(Box::new(LocalCommandDataSource::new(
                        command.clone(),
                        args.clone(),
                    )));
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
        for source in &self.sources {
            self.check_source(source.as_ref()).await;
        }
        Ok(())
    }

    async fn check_source(&self, source: &dyn ZpoolDataSource) {
        let source_name = source.name();
        info!("Checking source: {}", source_name);

        let json_data = match source.fetch().await {
            Ok(data) => data,
            Err(e) => {
                error!(
                    "Failed to fetch status from source '{}': {:?}",
                    source_name, e
                );
                return;
            }
        };

        let reports = match HealthChecker::check(&json_data) {
            Ok(reports) => reports,
            Err(e) => {
                error!(
                    "Failed to check health for source '{}': {:?}",
                    source_name, e
                );
                return;
            }
        };

        for report in reports {
            self.process_health_report(&report, &source_name).await;
        }
    }

    async fn process_health_report(&self, report: &health::HealthReport, source_name: &str) {
        let should_notify = !report.is_healthy || !self.config.settings.notify_on_error_only;

        if !report.is_healthy {
            warn!(
                "Pool '{}' from source '{}' is unhealthy: {}",
                report.pool_name, source_name, report.message
            );
        } else {
            info!(
                "Pool '{}' from source '{}' is healthy",
                report.pool_name, source_name
            );
        }

        if should_notify {
            self.send_notifications(report).await;
        }
    }

    async fn send_notifications(&self, report: &health::HealthReport) {
        for notifier in &self.notifiers {
            let notifier_name = notifier.name();

            match notifier.notify(report).await {
                Ok(_) => {
                    info!("Successfully sent notification via {}", notifier_name);
                }
                Err(e) => {
                    error!("Failed to send notification via {}: {:?}", notifier_name, e);
                }
            }
        }
    }

    async fn run(&self) -> Result<()> {
        let interval = Duration::from_secs(self.config.settings.check_interval);

        info!(
            "Starting ZWatch with {} data source(s) and {} notifier(s)",
            self.sources.len(),
            self.notifiers.len()
        );
        info!(
            "Check interval: {} seconds",
            self.config.settings.check_interval
        );

        // Perform initial check immediately
        if let Err(e) = self.check_and_notify().await {
            error!("Error during initial check cycle: {:?}", e);
        }

        loop {
            tokio::select! {
                _ = shutdown_signal() => {
                    info!("Received shutdown signal, exiting gracefully...");
                    break;
                }
                _ = tokio::time::sleep(interval) => {
                    if let Err(e) = self.check_and_notify().await {
                        error!("Error during check cycle: {:?}", e);
                    }
                }
            }
        }

        info!("ZWatch shutdown complete");
        Ok(())
    }
}

/// Wait for a shutdown signal (SIGTERM, SIGINT, or Ctrl+C)
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received SIGINT (Ctrl+C)");
        },
        _ = terminate => {
            info!("Received SIGTERM");
        },
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init { output }) => handle_init(&output),
        Some(Commands::Check { config }) => {
            let config_path = config.or(cli.config);
            handle_check(config_path).await
        }
        None => {
            // Default: run continuous monitoring
            handle_run(cli.config).await
        }
    }
}

fn init_logging() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_line_number(true)
        .init();
}

fn handle_init(output: &str) -> Result<()> {
    let config = Config::example();
    config.save(output)?;
    println!("Example configuration saved to {}", output);
    Ok(())
}

async fn handle_check(config_path: Option<String>) -> Result<()> {
    let config = load_config(config_path.as_ref())?;
    let watcher = ZWatch::new(config)?;
    watcher.check_and_notify().await
}

async fn handle_run(config_path: Option<String>) -> Result<()> {
    let config = load_config(config_path.as_ref())?;
    let watcher = ZWatch::new(config)?;
    watcher.run().await
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
