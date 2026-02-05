//! Bounty Challenge CLI
//!
//! Command-line interface for the Bounty Challenge.

mod client;
mod commands;
mod style;
mod wizard;

use clap::{Parser, Subcommand};
use std::net::IpAddr;
use style::*;

/// Validates that a string is a valid hostname or IP address for server binding.
///
/// Rejects:
/// - URLs (containing "://")
/// - Hostnames with ports embedded (containing ":")
/// - Invalid characters for hostnames
///
/// Accepts:
/// - Valid IPv4 addresses (e.g., "0.0.0.0", "127.0.0.1")
/// - Valid IPv6 addresses (e.g., "::1", "::0")
/// - Valid hostnames (e.g., "localhost", "my-server.local")
fn validate_server_host(s: &str) -> Result<String, String> {
    let s = s.trim();

    // Reject URLs
    if s.contains("://") {
        return Err(format!(
            "Invalid host '{}': URLs are not allowed. Use a hostname or IP address (e.g., '0.0.0.0', 'localhost').",
            s
        ));
    }

    // Reject embedded ports
    // Note: IPv6 addresses contain colons but are enclosed in brackets when ports are added
    if s.contains(':') && !s.parse::<IpAddr>().is_ok() {
        return Err(format!(
            "Invalid host '{}': Ports should be specified separately with --port. Use just the hostname or IP.",
            s
        ));
    }

    // Try parsing as IP address first
    if s.parse::<IpAddr>().is_ok() {
        return Ok(s.to_string());
    }

    // Validate as hostname per RFC 1123
    // - Labels separated by dots
    // - Each label: 1-63 chars, alphanumeric or hyphen, cannot start/end with hyphen
    // - Total length: max 253 characters
    if s.len() > 253 {
        return Err(format!(
            "Invalid host '{}': Hostname exceeds maximum length of 253 characters.",
            s
        ));
    }

    if s.is_empty() {
        return Err("Invalid host: Hostname cannot be empty.".to_string());
    }

    for label in s.split('.') {
        if label.is_empty() || label.len() > 63 {
            return Err(format!(
                "Invalid host '{}': Each hostname label must be 1-63 characters.",
                s
            ));
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err(format!(
                "Invalid host '{}': Hostname labels cannot start or end with a hyphen.",
                s
            ));
        }
        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(format!(
                "Invalid host '{}': Hostname contains invalid characters. Use only letters, numbers, and hyphens.",
                s
            ));
        }
    }

    Ok(s.to_string())
}

const BANNER: &str = r#"
  ██████╗  ██████╗ ██╗   ██╗███╗   ██╗████████╗██╗   ██╗
  ██╔══██╗██╔═══██╗██║   ██║████╗  ██║╚══██╔══╝╚██╗ ██╔╝
  ██████╔╝██║   ██║██║   ██║██╔██╗ ██║   ██║    ╚████╔╝ 
  ██╔══██╗██║   ██║██║   ██║██║╚██╗██║   ██║     ╚██╔╝  
  ██████╔╝╚██████╔╝╚██████╔╝██║ ╚████║   ██║      ██║   
  ╚═════╝  ╚═════╝  ╚═════╝ ╚═╝  ╚═══╝   ╚═╝      ╚═╝   
"#;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "bounty")]
#[command(author = "CortexLM")]
#[command(version)]
#[command(about = "Bounty Challenge - Earn rewards for finding bugs", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Platform RPC endpoint
    #[arg(
        short,
        long,
        env = "PLATFORM_URL",
        default_value = "https://chain.platform.network",
        global = true
    )]
    rpc: String,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive registration wizard - link GitHub to your hotkey (default)
    #[command(visible_aliases = ["w", "register", "r"])]
    Wizard,

    /// Run as server (for subnet operators)
    #[command(visible_alias = "s")]
    Server {
        /// Host to bind (hostname or IP address, e.g., "0.0.0.0", "localhost")
        #[arg(long, env = "CHALLENGE_HOST", default_value = "0.0.0.0", value_parser = validate_server_host)]
        host: String,

        /// Port to listen on
        #[arg(short, long, env = "CHALLENGE_PORT", default_value = "8080")]
        port: u16,

        /// PostgreSQL database URL
        #[arg(long, env = "DATABASE_URL")]
        database_url: String,
    },

    /// Run as validator (auto-discovers bounties)
    #[command(visible_alias = "v")]
    Validate {
        /// Platform server URL
        #[arg(
            long,
            env = "PLATFORM_URL",
            default_value = "https://chain.platform.network"
        )]
        platform: String,

        /// Validator hotkey
        #[arg(short = 'k', long, env = "VALIDATOR_HOTKEY")]
        hotkey: Option<String>,
    },

    /// View the leaderboard
    #[command(visible_alias = "lb")]
    Leaderboard {
        /// Number of entries to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Check your status and bounties
    #[command(visible_alias = "st")]
    Status {
        /// Your miner hotkey
        #[arg(short = 'k', long, env = "MINER_HOTKEY")]
        hotkey: String,
    },

    /// Show challenge configuration
    Config,

    /// Display system information for bug reports
    #[command(visible_alias = "i")]
    Info,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if cli.verbose {
        tracing_subscriber::fmt().with_env_filter("info").init();
    }

    // Default to wizard if no command specified
    let command = cli.command.unwrap_or(Commands::Wizard);

    let result = match command {
        Commands::Wizard => wizard::run_register_wizard(&cli.rpc).await,
        Commands::Server {
            host,
            port,
            database_url,
        } => {
            print_banner();
            commands::server::run(&host, port, &database_url).await
        }
        Commands::Validate { platform, hotkey } => commands::validate::run(&platform, hotkey).await,
        Commands::Leaderboard { limit } => commands::leaderboard::run(&cli.rpc, limit).await,
        Commands::Status { hotkey } => commands::status::run(&cli.rpc, &hotkey).await,
        Commands::Config => commands::config::run(&cli.rpc).await,
        Commands::Info => commands::info::run().await,
    };

    if let Err(e) = result {
        print_error(&format!("{}", e));
        std::process::exit(1);
    }
}

pub fn print_banner() {
    println!("{}", style_cyan(BANNER));
    println!(
        "  {} {}",
        style_dim("Bounty Challenge"),
        style_dim(&format!("v{}", VERSION))
    );
    println!();
}
