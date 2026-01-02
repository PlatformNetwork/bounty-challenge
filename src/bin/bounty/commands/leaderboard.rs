//! Leaderboard command

use crate::style::*;
use anyhow::{Context, Result};

pub async fn run(rpc: &str, limit: usize) -> Result<()> {
    print_header("Bounty Challenge Leaderboard");

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/leaderboard", rpc))
        .send()
        .await
        .context("Failed to connect to server")?;

    let result: serde_json::Value = response.json().await?;

    if let Some(error) = result.get("error") {
        print_error(&format!("Error: {}", error));
        return Ok(());
    }

    let leaderboard = result["leaderboard"].as_array();

    if let Some(entries) = leaderboard {
        if entries.is_empty() {
            print_info("No miners with valid bounties yet.");
            return Ok(());
        }

        println!();
        println!(
            "{:>4}  {:<18}  {:>12}  {:>8}  Hotkey",
            "Rank", "GitHub", "Valid Issues", "Score"
        );
        println!("{}", "─".repeat(75));

        for (i, entry) in entries.iter().take(limit).enumerate() {
            let github = entry["github_username"].as_str().unwrap_or("?");
            let issues = entry["valid_issues"].as_u64().unwrap_or(0);
            let score = entry["score"].as_f64().unwrap_or(0.0);
            let hotkey = entry["miner_hotkey"].as_str().unwrap_or("?");

            let rank = format!("#{}", i + 1);
            let hotkey_short = if hotkey.len() > 12 {
                format!("{}...", &hotkey[..12])
            } else {
                hotkey.to_string()
            };

            let rank_styled = if i == 0 {
                style_yellow(&rank)
            } else if i < 3 {
                style_cyan(&rank)
            } else {
                rank
            };

            println!(
                "{:>4}  {:<18}  {:>12}  {:>8.4}  {}",
                rank_styled,
                github,
                issues,
                score,
                style_dim(&hotkey_short)
            );
        }

        println!();
        println!("Total miners: {}", entries.len());
    }

    Ok(())
}
