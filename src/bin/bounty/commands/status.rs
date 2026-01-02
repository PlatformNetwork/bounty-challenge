//! Status command - check miner status

use crate::style::*;
use anyhow::{Context, Result};

pub async fn run(rpc: &str, hotkey: &str) -> Result<()> {
    print_header("Miner Status");

    println!("Hotkey: {}...{}", &hotkey[..8], &hotkey[hotkey.len() - 4..]);
    println!();

    let client = reqwest::Client::new();

    // Get leaderboard and find this miner
    let response = client
        .get(format!("{}/leaderboard", rpc))
        .send()
        .await
        .context("Failed to connect to server")?;

    let result: serde_json::Value = response.json().await?;

    if let Some(leaderboard) = result["leaderboard"].as_array() {
        let miner = leaderboard
            .iter()
            .find(|e| e["miner_hotkey"].as_str() == Some(hotkey));

        if let Some(entry) = miner {
            let github = entry["github_username"].as_str().unwrap_or("?");
            let issues = entry["valid_issues"].as_u64().unwrap_or(0);
            let score = entry["score"].as_f64().unwrap_or(0.0);
            let rank = leaderboard
                .iter()
                .position(|e| e["miner_hotkey"].as_str() == Some(hotkey))
                .map(|i| i + 1)
                .unwrap_or(0);

            print_success("Miner found!");
            println!();
            println!("GitHub Username:  @{}", style_cyan(github));
            println!("Valid Issues:     {}", style_bold(&issues.to_string()));
            println!(
                "Current Score:    {}",
                style_green(&format!("{:.4}", score))
            );
            println!("Rank:             #{} of {}", rank, leaderboard.len());
        } else {
            print_warning("Miner not found in leaderboard.");
            println!();
            println!("This could mean:");
            println!("  - You haven't registered your GitHub account yet");
            println!("  - You haven't claimed any valid bounties yet");
            println!();
            println!("To register, run:");
            println!("  bounty register --hotkey {}", hotkey);
        }
    }

    Ok(())
}
