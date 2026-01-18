//! Bounty Challenge CLI
//!
//! Command-line interface for the Bounty Challenge.

mod client;
mod commands;
mod style;
mod wizard;

use clap::{Parser, Subcommand};
use style::*;

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
        /// Host to bind
        #[arg(long, env = "CHALLENGE_HOST", default_value = "0.0.0.0")]
        host: String,

        /// Port to listen on
        #[arg(short, long, env = "CHALLENGE_PORT", default_value = "8080")]
        port: u16,

        /// Database path (SQLite for local cache)
        #[arg(long, env = "BOUNTY_DB_PATH", default_value = "bounty.db")]
        db: String,
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
        #[arg(short, long, env = "VALIDATOR_HOTKEY")]
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
        #[arg(short, long, env = "MINER_HOTKEY")]
        hotkey: String,
    },

    /// Show challenge configuration
    Config,
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
        Commands::Server { host, port, db } => {
            print_banner();
            commands::server::run(&host, port, &db).await
        }
        Commands::Validate { platform, hotkey } => commands::validate::run(&platform, hotkey).await,
        Commands::Leaderboard { limit } => commands::leaderboard::run(&cli.rpc, limit).await,
        Commands::Status { hotkey } => commands::status::run(&cli.rpc, &hotkey).await,
        Commands::Config => commands::config::run(&cli.rpc).await,
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
