use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde_json::Value;

const DEFAULT_RPC_URL: &str = "http://localhost:8080";
const CHALLENGE_ID: &str = "bounty-challenge";

#[derive(Parser)]
#[command(name = "bounty-cli")]
#[command(about = "Bounty Challenge CLI — interact with the bounty challenge on Platform Network")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Platform validator RPC URL
    #[arg(long, global = true, default_value = DEFAULT_RPC_URL, env = "BOUNTY_RPC_URL")]
    rpc_url: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Show the current leaderboard
    Leaderboard {
        /// Maximum number of entries to display
        #[arg(long, short, default_value = "50")]
        limit: usize,
    },

    /// Register a GitHub username with a hotkey
    Register {
        /// SS58-encoded hotkey
        #[arg(long)]
        hotkey: String,

        /// GitHub username to associate
        #[arg(long)]
        github: String,

        /// Hex-encoded sr25519 signature of "register_github:{username_lowercase}:{timestamp}"
        #[arg(long)]
        signature: String,

        /// Unix timestamp used when creating the signature
        #[arg(long)]
        timestamp: i64,
    },

    /// Check status for a specific hotkey
    Status {
        /// SS58-encoded hotkey to look up
        #[arg(long)]
        hotkey: String,
    },

    /// Show challenge statistics
    Stats,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let rpc_url = cli.rpc_url.trim_end_matches('/').to_string();

    match cli.command {
        Commands::Leaderboard { limit } => cmd_leaderboard(&rpc_url, limit).await,
        Commands::Register {
            hotkey,
            github,
            signature,
            timestamp,
        } => cmd_register(&rpc_url, &hotkey, &github, &signature, timestamp).await,
        Commands::Status { hotkey } => cmd_status(&rpc_url, &hotkey).await,
        Commands::Stats => cmd_stats(&rpc_url).await,
    }
}

async fn rpc_call(rpc_url: &str, method: &str, path: &str, body: Option<Value>) -> Result<Value> {
    let client = reqwest::Client::new();

    let mut params = serde_json::json!({
        "challengeId": CHALLENGE_ID,
        "method": method,
        "path": path,
    });

    if let Some(b) = body {
        params["body"] = b;
    }

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "challenge_call",
        "params": params,
        "id": 1,
    });

    let response = client
        .post(format!("{}/rpc", rpc_url))
        .json(&request)
        .send()
        .await
        .context("Failed to connect to validator RPC")?;

    let status = response.status();
    let json: Value = response
        .json()
        .await
        .context("Failed to parse RPC response")?;

    if let Some(error) = json.get("error") {
        let msg = error
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown RPC error");
        anyhow::bail!("RPC error (HTTP {}): {}", status, msg);
    }

    let result = json.get("result").cloned().unwrap_or(Value::Null);

    Ok(result)
}

async fn cmd_leaderboard(rpc_url: &str, limit: usize) -> Result<()> {
    let result = rpc_call(rpc_url, "GET", "/leaderboard", None).await?;

    let body = result.get("body").unwrap_or(&result);

    let entries = match body.as_array() {
        Some(arr) => arr,
        None => {
            println!("No leaderboard data available.");
            return Ok(());
        }
    };

    if entries.is_empty() {
        println!("Leaderboard is empty — no miners registered yet.");
        return Ok(());
    }

    println!(
        "{:<6} {:<15} {:<20} {:<12} {:<8} {:<8} {:<10} {:<10}",
        "Rank", "Hotkey", "GitHub", "Net Points", "Valid", "Invalid", "Stars", "Weight"
    );
    println!("{}", "-".repeat(95));

    for entry in entries.iter().take(limit) {
        let rank = entry.get("rank").and_then(|v| v.as_u64()).unwrap_or(0);
        let hotkey = entry.get("hotkey").and_then(|v| v.as_str()).unwrap_or("?");
        let hotkey_short = if hotkey.len() > 12 {
            format!("{}…", &hotkey[..12])
        } else {
            hotkey.to_string()
        };
        let github = entry
            .get("github_username")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let net_points = entry
            .get("net_points")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let valid = entry
            .get("valid_issues")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let invalid = entry
            .get("invalid_issues")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let stars = entry
            .get("star_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let weight = entry.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);

        println!(
            "{:<6} {:<15} {:<20} {:<12.2} {:<8} {:<8} {:<10} {:<10.4}",
            rank, hotkey_short, github, net_points, valid, invalid, stars, weight
        );
    }

    if entries.len() > limit {
        println!(
            "\n… showing {} of {} entries (use --limit to see more)",
            limit,
            entries.len()
        );
    }

    Ok(())
}

async fn cmd_register(
    rpc_url: &str,
    hotkey: &str,
    github: &str,
    signature: &str,
    timestamp: i64,
) -> Result<()> {
    let body = serde_json::json!({
        "hotkey": hotkey,
        "github_username": github,
        "signature": signature,
        "timestamp": timestamp,
    });

    let result = rpc_call(rpc_url, "POST", "/register", Some(body)).await?;

    let response_body = result.get("body").unwrap_or(&result);

    let success = response_body.as_bool().unwrap_or(false);

    if success {
        println!(
            "✅ Successfully registered GitHub user '{}' with hotkey {}",
            github, hotkey
        );
    } else {
        println!("❌ Registration failed. Check that:");
        println!("   • The hotkey is valid and registered on the subnet");
        println!(
            "   • The signature is correct (sign: register_github:{}:{})",
            github.to_lowercase(),
            timestamp
        );
        println!("   • The timestamp is within 5 minutes of server time");
    }

    Ok(())
}

async fn cmd_status(rpc_url: &str, hotkey: &str) -> Result<()> {
    let path = format!("/status/{}", hotkey);
    let result = rpc_call(rpc_url, "GET", &path, None).await?;

    let body = result.get("body").unwrap_or(&result);

    let registered = body
        .get("registered")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !registered {
        println!("Hotkey {} is not registered.", hotkey);
        println!("\nUse 'bounty-cli register' to register a GitHub username.");
        return Ok(());
    }

    let github = body
        .get("github_username")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let valid = body
        .get("valid_issues_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let invalid = body
        .get("invalid_issues_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let weight = body.get("weight").and_then(|v| v.as_f64()).unwrap_or(0.0);

    let balance = body.get("balance");
    let duplicates = balance
        .and_then(|b| b.get("duplicate_count"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let stars = balance
        .and_then(|b| b.get("star_count"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let penalized = balance
        .and_then(|b| b.get("is_penalized"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    println!("Miner Status");
    println!("{}", "=".repeat(40));
    println!("Hotkey:           {}", hotkey);
    println!("GitHub:           {}", github);
    println!("Registered:       yes");
    println!();
    println!("Valid Issues:     {}", valid);
    println!("Invalid Issues:   {}", invalid);
    println!("Duplicate Issues: {}", duplicates);
    println!("Starred Repos:    {}", stars);
    println!();
    println!("Weight:           {:.4}", weight);
    if penalized {
        println!("Status:           ⚠️  PENALIZED (weight = 0)");
    } else {
        println!("Status:           ✅ Active");
    }

    Ok(())
}

async fn cmd_stats(rpc_url: &str) -> Result<()> {
    let result = rpc_call(rpc_url, "GET", "/stats", None).await?;

    let body = result.get("body").unwrap_or(&result);

    let total_bounties = body
        .get("total_bounties")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let active_miners = body
        .get("active_miners")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let validator_count = body
        .get("validator_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let total_issues = body
        .get("total_issues")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    println!("Bounty Challenge Statistics");
    println!("{}", "=".repeat(40));
    println!("Total Bounties:   {}", total_bounties);
    println!("Active Miners:    {}", active_miners);
    println!("Validators:       {}", validator_count);
    println!("Total Issues:     {}", total_issues);

    Ok(())
}
