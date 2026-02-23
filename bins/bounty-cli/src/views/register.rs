use anyhow::{Context, Result};
use console::style;
use dialoguer::{Input, Password};
use sp_core::{crypto::Pair as PairTrait, sr25519::Pair};

use crate::rpc::rpc_call_auth;

pub async fn run(rpc_url: &str) -> Result<()> {
    println!("\n{}", style("Register GitHub Username").cyan().bold());
    println!("{}\n", style("─".repeat(40)).dim());

    let github: String = Input::new()
        .with_prompt("GitHub username")
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

    println!("{}", style("Deriving sr25519 keypair...").dim());

    let (pair, _seed) = Pair::from_phrase(mnemonic, None).context("Invalid mnemonic phrase")?;

    let hotkey_ss58 = sp_core::crypto::Ss58Codec::to_ss58check(&pair.public());

    println!(
        "  {} {}",
        style("Hotkey:").dim(),
        style(&hotkey_ss58).green()
    );

    // Body just contains the github username
    // Authentication is done via X-Hotkey, X-Signature, X-Nonce headers
    let body = serde_json::json!({
        "github_username": github,
    });

    println!("{}", style("Sending authenticated registration...").dim());

    let result = rpc_call_auth(rpc_url, "POST", "/register", Some(body), &pair).await?;
    let response_body = result.get("body").unwrap_or(&result);

    // Check for success
    let success = response_body
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or_else(|| response_body.as_bool().unwrap_or(false));

    if success {
        println!(
            "\n{}",
            style(format!(
                "Successfully registered '{}' with hotkey {}",
                github, hotkey_ss58
            ))
            .green()
            .bold()
        );
    } else {
        let error = response_body
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error");
        println!("\n{}", style("Registration failed.").red().bold());
        println!("  Error: {}", error);
    }

    println!();
    Ok(())
}
