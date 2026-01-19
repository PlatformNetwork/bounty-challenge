//! Registration Wizard - Interactive GitHub account linking
//!
//! Guides the user through registering their GitHub username with their miner hotkey.
//! Uses sr25519 signature to prove ownership of the hotkey.

use anyhow::Result;
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password};
use indicatif::{ProgressBar, ProgressStyle};
use sp_core::{sr25519, Pair};
use std::time::Duration;

use crate::client::{BountyClient, RegisterRequest};

/// SS58 prefix for Bittensor (same as term-challenge)
const SS58_PREFIX: u16 = 42;

pub async fn run_register_wizard(platform_url: &str) -> Result<()> {
    print_banner();
    println!();
    println!(
        "{}",
        style("  Interactive Registration Wizard").cyan().bold()
    );
    println!(
        "  {}",
        style("Link your GitHub account to your miner hotkey").dim()
    );
    println!();

    // Step 1: Enter miner secret key
    println!("  {}", style("Step 1: Enter Miner Key").bold());
    println!(
        "  {}",
        style("(64-char hex seed or 12+ word mnemonic)").dim()
    );
    println!();

    let (signing_key, hotkey) = enter_miner_key()?;

    println!(
        "  {} Hotkey: {}...{}",
        style("✓").green(),
        style(&hotkey[..8]).cyan(),
        style(&hotkey[hotkey.len() - 4..]).cyan()
    );

    // Step 2: Enter GitHub username
    println!();
    println!("  {}", style("Step 2: Enter GitHub Username").bold());
    println!(
        "  {}",
        style("The username you use to create issues").dim()
    );
    println!();

    let github_username: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("  GitHub username")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.is_empty() {
                return Err("Username cannot be empty");
            }
            if input.len() > 39 {
                return Err("GitHub usernames are max 39 characters");
            }
            if !input
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-')
            {
                return Err("Username can only contain alphanumeric and hyphens");
            }
            if input.starts_with('-') || input.ends_with('-') {
                return Err("Username cannot start or end with hyphen");
            }
            Ok(())
        })
        .interact_text()?;

    println!(
        "  {} GitHub: @{}",
        style("✓").green(),
        style(&github_username).cyan()
    );

    // Step 3: Review and confirm
    println!();
    println!("  {}", style("Review Registration").bold());
    println!("  {}", style("─".repeat(40)).dim());
    println!();
    println!(
        "  Hotkey:   {}...{}",
        &hotkey[..12],
        &hotkey[hotkey.len() - 4..]
    );
    println!("  GitHub:   @{}", style(&github_username).cyan());
    println!();

    let confirmed = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("  Register this GitHub account?")
        .default(true)
        .interact()?;

    if !confirmed {
        println!();
        println!("  {} Registration cancelled", style("✗").red());
        return Ok(());
    }

    // Step 4: Sign and submit
    println!();
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("  {spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Signing registration request...");
    pb.enable_steady_tick(Duration::from_millis(80));

    // Create signature
    let timestamp = chrono::Utc::now().timestamp();
    let message = format!("register_github:{}:{}", github_username.to_lowercase(), timestamp);
    let signature = signing_key.sign(message.as_bytes());
    let signature_hex = hex::encode(signature.0);

    pb.set_message("Submitting to network...");

    // Submit to bridge
    let client = BountyClient::new(platform_url);
    let request = RegisterRequest {
        hotkey: hotkey.clone(),
        github_username: github_username.clone(),
        signature: signature_hex,
        timestamp,
    };

    let result = client.register(&request).await;
    pb.finish_and_clear();

    match result {
        Ok(response) => {
            if response.success {
                println!();
                println!("  {}", style("═".repeat(50)).dim());
                println!();
                println!(
                    "  {} Registration successful!",
                    style("✓").green().bold()
                );
                println!();
                println!(
                    "  Your GitHub account {} is now linked.",
                    style(format!("@{}", github_username)).cyan()
                );
                println!();
                println!("  {}", style("Next steps:").bold());
                println!(
                    "    1. Create issues on {}",
                    style("https://github.com/PlatformNetwork/bounty-challenge").cyan()
                );
                println!(
                    "    2. Wait for maintainers to add the '{}' label",
                    style("valid").green()
                );
                println!("    3. Rewards are credited automatically!");
                println!();
                println!("  Check your status:");
                println!(
                    "    {}",
                    style(format!("bounty status --hotkey {}", &hotkey[..16])).yellow()
                );
                println!();
            } else {
                let error = response.error.unwrap_or_else(|| "Unknown error".to_string());
                println!();
                println!("  {} Registration failed: {}", style("✗").red(), error);
            }
        }
        Err(e) => {
            println!();
            println!("  {} Error: {}", style("✗").red(), e);
            println!();
            println!("  Make sure the platform server is running and accessible.");
        }
    }

    Ok(())
}

fn print_banner() {
    println!(
        r#"
  {}
  {}
  {}
  {}
  {}
  {}"#,
        style("██████╗  ██████╗ ██╗   ██╗███╗   ██╗████████╗██╗   ██╗").cyan(),
        style("██╔══██╗██╔═══██╗██║   ██║████╗  ██║╚══██╔══╝╚██╗ ██╔╝").cyan(),
        style("██████╔╝██║   ██║██║   ██║██╔██╗ ██║   ██║    ╚████╔╝ ").cyan(),
        style("██╔══██╗██║   ██║██║   ██║██║╚██╗██║   ██║     ╚██╔╝  ").cyan(),
        style("██████╔╝╚██████╔╝╚██████╔╝██║ ╚████║   ██║      ██║   ").cyan(),
        style("╚═════╝  ╚═════╝  ╚═════╝ ╚═╝  ╚═══╝   ╚═╝      ╚═╝   ").cyan(),
    );
}

fn enter_miner_key() -> Result<(sr25519::Pair, String)> {
    let key: String = Password::with_theme(&ColorfulTheme::default())
        .with_prompt("  Miner secret key")
        .interact()?;

    parse_miner_key(&key)
}

fn parse_miner_key(key: &str) -> Result<(sr25519::Pair, String)> {
    let key = key.trim();
    let key = key.strip_prefix("0x").unwrap_or(key);
    
    // Try hex seed first (64 chars = 32 bytes)
    if key.len() == 64 {
        if let Ok(bytes) = hex::decode(key) {
            if bytes.len() == 32 {
                let mut seed = [0u8; 32];
                seed.copy_from_slice(&bytes);
                let pair = sr25519::Pair::from_seed(&seed);
                let hotkey_ss58 = encode_ss58(&pair.public().0);
                return Ok((pair, hotkey_ss58));
            }
        }
    }

    // Try SURI format (supports derivation paths like "mnemonic//hard/soft")
    // This is the most flexible format used by subkey and substrate tools
    if let Ok((pair, _)) = sr25519::Pair::from_string_with_seed(key, None) {
        let hotkey_ss58 = encode_ss58(&pair.public().0);
        return Ok((pair, hotkey_ss58));
    }

    // Try mnemonic phrase without derivation
    if let Ok((pair, _)) = sr25519::Pair::from_phrase(key, None) {
        let hotkey_ss58 = encode_ss58(&pair.public().0);
        return Ok((pair, hotkey_ss58));
    }

    anyhow::bail!(
        "Invalid key format. Expected:\n  - 64-char hex seed\n  - 12+ word mnemonic\n  - SURI (e.g., //Alice)"
    );
}

/// Encode bytes to SS58 format with Bittensor prefix
fn encode_ss58(public_key: &[u8; 32]) -> String {
    use blake2::{Blake2b512, Digest};

    let prefix = SS58_PREFIX;
    
    // Build the payload
    let mut payload = Vec::with_capacity(35);
    if prefix < 64 {
        payload.push(prefix as u8);
    } else {
        payload.push(((prefix & 0x00FC) >> 2) as u8 | 0x40);
        payload.push(((prefix >> 8) as u8) | ((prefix & 0x0003) as u8) << 6);
    }
    payload.extend_from_slice(public_key);

    // Calculate checksum using Blake2b-512 (Substrate standard)
    let mut hasher = Blake2b512::new();
    hasher.update(b"SS58PRE");
    hasher.update(&payload);
    let hash = hasher.finalize();

    payload.extend_from_slice(&hash[..2]);

    // Base58 encode
    bs58::encode(payload).into_string()
}
