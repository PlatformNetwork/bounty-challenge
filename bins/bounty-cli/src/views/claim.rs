use anyhow::{Context, Result};
use console::style;
use dialoguer::{Input, Password};
use sp_core::{crypto::Pair as PairTrait, sr25519::Pair};

use crate::rpc::rpc_call;

pub async fn run(rpc_url: &str) -> Result<()> {
    println!("\n{}", style("Claim Bounty").cyan().bold());
    println!("{}\n", style("─".repeat(40)).dim());

    let issue_url: String = Input::new()
        .with_prompt("GitHub issue URL")
        .interact_text()?;

    let mnemonic: String = Password::new()
        .with_prompt("Enter your 24-word mnemonic (hidden)")
        .interact()?;

    let mnemonic = mnemonic.trim();
    let words: Vec<&str> = mnemonic.split_whitespace().collect();
    if words.len() != 12 && words.len() != 24 {
        anyhow::bail!(
            "Expected 12 or 24 words, got {}. Check your mnemonic.",
            words.len()
        );
    }

    let (pair, _seed) =
        Pair::from_phrase(mnemonic, None).context("Invalid mnemonic phrase")?;

    let hotkey = sp_core::crypto::Ss58Codec::to_ss58check(&pair.public());

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;

    let message = format!("claim_bounty:{}:{}:{}", hotkey, issue_url, timestamp);
    let signature = pair.sign(message.as_bytes());
    let sig_hex = hex::encode(signature.0);

    println!(
        "  {} {}",
        style("Hotkey:").dim(),
        style(&hotkey).green()
    );
    println!(
        "  {} {}",
        style("Issue:").dim(),
        style(&issue_url).yellow()
    );

    let body = serde_json::json!({
        "hotkey": hotkey,
        "issue_url": issue_url,
        "signature": sig_hex,
        "timestamp": timestamp,
    });

    println!("{}", style("Submitting claim...").dim());

    let result = rpc_call(rpc_url, "POST", "/claim", Some(body)).await?;
    let response_body = result.get("body").unwrap_or(&result);

    if response_body.as_bool().unwrap_or(false) {
        println!(
            "\n{}",
            style("Claim submitted successfully!").green().bold()
        );
    } else {
        let msg = response_body
            .as_str()
            .unwrap_or("Check that the issue is valid and closed with the 'valid' label.");
        println!("\n{}", style("Claim failed.").red().bold());
        println!("  {}", msg);
    }

    println!();
    Ok(())
}
