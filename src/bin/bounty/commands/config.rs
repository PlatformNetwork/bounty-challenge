//! Config command - show challenge configuration

use crate::style::*;
use anyhow::{Context, Result};

pub async fn run(rpc: &str) -> Result<()> {
    print_header("Challenge Configuration");

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/config", rpc))
        .send()
        .await
        .context("Failed to connect to server")?;

    let config: serde_json::Value = response.json().await?;

    println!();
    println!(
        "Challenge ID:     {}",
        style_cyan(config["challenge_id"].as_str().unwrap_or("?"))
    );
    println!(
        "Name:             {}",
        config["name"].as_str().unwrap_or("?")
    );
    println!(
        "Version:          {}",
        config["version"].as_str().unwrap_or("?")
    );

    if let Some(features) = config["features"].as_array() {
        println!(
            "Features:         {}",
            features
                .iter()
                .filter_map(|f| f.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    if let Some(limits) = config.get("limits") {
        println!();
        println!("{}", style_bold("Limits:"));
        if let Some(size) = limits["max_submission_size"].as_u64() {
            println!("  Max submission:   {} bytes", size);
        }
        if let Some(time) = limits["max_evaluation_time"].as_u64() {
            println!("  Max eval time:    {}s", time);
        }
    }

    println!();
    println!("{}", style_bold("Target Repository:"));
    println!("  https://github.com/CortexLM/fabric");
    println!();
    println!("{}", style_bold("Valid Issue Criteria:"));
    println!("  - Issue must be closed");
    println!("  - Issue must have '{}' label", style_green("valid"));
    println!("  - You must be the issue author");
    println!("  - Issue must not be already claimed");

    Ok(())
}
