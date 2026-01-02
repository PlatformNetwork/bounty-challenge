//! Register command - link GitHub account via Device Flow

use crate::style::*;
use anyhow::{Context, Result};

pub async fn run(rpc: &str, hotkey: &str) -> Result<()> {
    print_header("GitHub Registration");

    // Validate hotkey
    if !bounty_challenge::auth::is_valid_ss58_hotkey(hotkey) {
        print_error("Invalid hotkey format. Must be SS58 encoded.");
        return Ok(());
    }

    println!("Hotkey: {}...{}", &hotkey[..8], &hotkey[hotkey.len() - 4..]);
    println!();

    // Initialize GitHub Device Auth
    let auth = bounty_challenge::github_oauth::GitHubDeviceAuth::from_env()
        .context("GitHub OAuth not configured. Set GITHUB_CLIENT_ID env var.")?;

    print_info("Requesting authorization from GitHub...");
    println!();

    // Step 1: Get device code
    let device_code = auth.request_device_code().await?;

    // Display instructions to user
    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│                                                         │");
    println!("│  To link your GitHub account:                          │");
    println!("│                                                         │");
    println!(
        "│  1. Go to: {}       │",
        style_cyan(&device_code.verification_uri)
    );
    println!("│                                                         │");
    println!(
        "│  2. Enter code: {}                              │",
        style_bold(&device_code.user_code)
    );
    println!("│                                                         │");
    println!("│  3. Authorize the application                          │");
    println!("│                                                         │");
    println!("└─────────────────────────────────────────────────────────┘");
    println!();

    // Try to open browser
    if open::that(&device_code.verification_uri).is_ok() {
        print_info("Browser opened automatically.");
    }

    println!();
    print_info("Waiting for authorization...");

    // Step 2: Poll for token
    let access_token = auth.poll_for_token(&device_code).await?;

    // Step 3: Get user info
    let user = auth.get_user(&access_token).await?;

    println!();
    print_success(&format!("GitHub account verified: @{}", user.login));

    // Step 4: Register with server
    print_info("Registering with Bounty Challenge server...");

    let client = reqwest::Client::new();
    let request = serde_json::json!({
        "request_id": format!("reg-{}", chrono::Utc::now().timestamp()),
        "submission_id": format!("sub-{}", chrono::Utc::now().timestamp()),
        "participant_id": hotkey,
        "epoch": 1,
        "data": {
            "action": "register",
            "github_username": user.login
        }
    });

    let response = client
        .post(format!("{}/evaluate", rpc))
        .json(&request)
        .send()
        .await
        .context("Failed to connect to server")?;

    let result: serde_json::Value = response.json().await?;

    if result["success"].as_bool().unwrap_or(false) {
        println!();
        print_success("Registration complete!");
        println!();
        println!(
            "Your GitHub account @{} is now linked to your hotkey.",
            style_cyan(&user.login)
        );
        println!();
        println!("Next steps:");
        println!(
            "  1. Create issues on {}",
            style_cyan("https://github.com/CortexLM/fabric/issues")
        );
        println!(
            "  2. Wait for maintainers to validate with '{}' label",
            style_green("valid")
        );
        println!("  3. Bounties are credited automatically!");
        println!();
        println!("Check your status anytime:");
        println!("  bounty status --hotkey {}", &hotkey[..16]);
    } else {
        let error = result["error"].as_str().unwrap_or("Unknown error");
        print_error(&format!("Registration failed: {}", error));
    }

    Ok(())
}
