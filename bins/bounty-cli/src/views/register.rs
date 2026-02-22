use anyhow::{Context, Result};
use console::style;
use dialoguer::{Input, Password};
use sp_core::{crypto::Pair as PairTrait, sr25519::Pair};

use crate::rpc::rpc_call;

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

    let (pair, _seed) =
        Pair::from_phrase(mnemonic, None).context("Invalid mnemonic phrase")?;

    let hotkey = sp_core::crypto::Ss58Codec::to_ss58check(&pair.public());

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;

    let message = format!(
        "register_github:{}:{}",
        github.to_lowercase(),
        timestamp
    );

    let signature = pair.sign(message.as_bytes());
    let sig_hex = hex::encode(signature.0);

    println!(
        "  {} {}",
        style("Hotkey:").dim(),
        style(&hotkey).green()
    );
    println!(
        "  {} {}",
        style("Message:").dim(),
        style(&message).yellow()
    );

    let body = serde_json::json!({
        "hotkey": hotkey,
        "github_username": github,
        "signature": sig_hex,
        "timestamp": timestamp,
    });

    println!("{}", style("Sending registration...").dim());

    let result = rpc_call(rpc_url, "POST", "/register", Some(body)).await?;
    let response_body = result.get("body").unwrap_or(&result);
    let success = response_body.as_bool().unwrap_or(false);

    if success {
        println!(
            "\n{}",
            style(format!(
                "Successfully registered '{}' with hotkey {}",
                github, hotkey
            ))
            .green()
            .bold()
        );
    } else {
        println!("\n{}", style("Registration failed.").red().bold());
        println!("  Check that the hotkey is registered on the subnet");
        println!("  and the timestamp is within 5 minutes of server time.");
    }

    println!();
    Ok(())
}
