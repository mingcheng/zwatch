# ZWatch - ZFS Pool Monitoring & Alerting System

A flexible monitoring and alerting system for ZFS storage pools with support for multiple data sources and notification channels.

## Features

### Multiple Data Sources

- **Local Command** - Execute `zpool status -j` directly on the local system
- **SSH Remote** - Fetch ZFS status from remote servers via SSH
- **File Reader** - Read status from JSON files (for testing or offline analysis)

### Multiple Notification Channels

- **Telegram** - Send alerts via Telegram Bot API
- **Webhook** - Send HTTP POST requests to custom webhooks
- **Bark** - iOS Bark push notification service
- **Console** - Console output (for debugging)

### Health Monitoring

- Automatic pool state detection (ONLINE/DEGRADED/FAULTED, etc.)
- Device error detection (read, write, checksum errors)
- Scrub/resilver error detection
- Recursive vdev health checks

## Installation

```bash
cargo build --release
```

## Usage

### Generate Example Configuration

```bash
zwatch init
```

This creates a `zwatch.toml` configuration file.

### Edit Configuration

```toml
# Data source configuration
[[data_sources]]
type = "local"
name = "localhost"
command = "zpool"
args = ["status", "-j"]

[[data_sources]]
type = "ssh"
name = "remote-server"
host = "192.168.1.100"
user = "root"
keyfile = "/home/user/.ssh/id_rsa"

# Notification configuration
[[notifiers]]
type = "telegram"
bot_token = "YOUR_BOT_TOKEN"
chat_id = "YOUR_CHAT_ID"

[[notifiers]]
type = "webhook"
url = "https://your-webhook-url.com/endpoint"

# Global settings
[settings]
check_interval = 300  # Check interval in seconds
notify_on_error_only = true  # Only send notifications on errors
```

### Run Monitoring

- Continuous monitoring (default): `zwatch [--config PATH]`
- One-time check: `zwatch check [--config PATH]`
- Generate example config: `zwatch init [--output FILE]`

For more details, run: `zwatch --help`.

#### Notes

- Platform: Local command data source works on Linux/FreeBSD only; on macOS use SSH or File data sources.
- Config discovery: If `--config` is omitted, ZWatch loads `zwatch.toml` in the current directory when present; otherwise uses built-in defaults.
- Logging: Control verbosity via `RUST_LOG`, e.g. `RUST_LOG=info zwatch` or `RUST_LOG=debug zwatch`.

## Extension Development

### Adding a New Data Source

Implement the `ZpoolDataSource` trait:

```rust
use async_trait::async_trait;

pub struct CustomDataSource {
    // Custom fields
}

#[async_trait]
impl ZpoolDataSource for CustomDataSource {
    async fn fetch(&self) -> Result<String> {
        // Implementation to fetch and return JSON string
    }

    fn name(&self) -> String {
        // Return data source name
    }
}
```

### Adding a New Notifier

Implement the `Notifier` trait:

```rust
use async_trait::async_trait;

pub struct CustomNotifier {
    // Custom fields
}

#[async_trait]
impl Notifier for CustomNotifier {
    async fn notify(&self, report: &HealthReport) -> Result<()> {
        // Implementation to send notification
    }

    fn name(&self) -> String {
        // Return notifier name
    }
}
```

## Alert Message Examples

### Healthy State

```
✅ ZFS Pool 'tank' is ONLINE and healthy
```

### Degraded State

```
⚠️ ZFS Pool 'tank' has issues!
State: DEGRADED
Pool Errors: 0
Scan Errors: 0

Device Errors:
- sda1: state=DEGRADED, read=0, write=5, checksum=0
- sdb1: state=OFFLINE, read=0, write=0, checksum=0

Details: Pool state is DEGRADED; 2 devices have errors
```

## License

This project is under the MIT License, for more details see the [LICENSE](./LICENSE) file.
