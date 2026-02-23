use anyhow::{Context, Result};
use console::style;
use dialoguer::{Input, Password};
use sp_core::{crypto::Pair as PairTrait, sr25519::Pair};

use crate::rpc::rpc_call_auth;

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

    let (pair, _seed) = Pair::from_phrase(mnemonic, None).context("Invalid mnemonic phrase")?;

    let hotkey_ss58 = sp_core::crypto::Ss58Codec::to_ss58check(&pair.public());

    println!(
        "  {} {}",
        style("Hotkey:").dim(),
        style(&hotkey_ss58).green()
    );
    println!("  {} {}", style("Issue:").dim(), style(&issue_url).yellow());

    // Body just contains the issue URL
    // Authentication is done via X-Hotkey, X-Signature, X-Nonce headers
    let body = serde_json::json!({
        "issue_url": issue_url,
    });

    println!("{}", style("Submitting authenticated claim...").dim());

    let result = rpc_call_auth(rpc_url, "POST", "/claim", Some(body), &pair).await?;
    let response_body = result.get("body").unwrap_or(&result);

    // ClaimResult has: claimed, rejected, total_valid, score
    let claimed = response_body
        .get("claimed")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let rejected = response_body
        .get("rejected")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    if claimed > 0 {
        println!(
            "\n{}",
            style("Claim submitted successfully!").green().bold()
        );
        println!("  Claimed: {}", claimed);
        if rejected > 0 {
            println!("  Rejected: {}", rejected);
        }
    } else if let Some(error) = response_body.get("error").and_then(|v| v.as_str()) {
        println!("\n{}", style("Claim failed.").red().bold());
        println!("  Error: {}", error);
    } else {
        println!("\n{}", style("Claim failed.").red().bold());
        if rejected > 0 {
            println!("  Rejected: {}", rejected);
        }
        println!("  Check that the issue has both 'ide' and 'valid' labels.");
    }

    println!();
    Ok(())
}
