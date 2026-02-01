//! Status command - check miner status

use crate::style::*;
use anyhow::Result;

pub async fn run(rpc: &str, hotkey: &str) -> Result<()> {
    print_header("Miner Status");

    println!("Hotkey: {}", crate::style::truncate_hotkey(hotkey));
    println!();

    // Use BountyClient for API consistency
    let client = crate::client::BountyClient::new(rpc);

    match client.get_status(hotkey).await {
        Ok(status) => {
            if status.registered {
                print_success("Miner registered!");
                println!();
                println!(
                    "GitHub Username:  @{}",
                    style_cyan(status.github_username.as_deref().unwrap_or("?"))
                );
                println!(
                    "Valid Issues:     {}",
                    style_bold(&status.valid_issues_count.unwrap_or(0).to_string())
                );
                if let Some(invalid) = status.invalid_issues_count {
                    println!(
                        "Invalid Issues:   {}",
                        if invalid > 0 {
                            style_yellow(&invalid.to_string())
                        } else {
                            style_dim(&invalid.to_string())
                        }
                    );
                }
                if let Some(balance) = status.balance {
                    println!(
                        "Balance:          {}",
                        if balance < 0 {
                            style_red(&balance.to_string())
                        } else {
                            style_green(&format!("+{}", balance))
                        }
                    );
                }
                println!(
                    "Current Weight:   {}",
                    style_green(&format!("{:.2}%", status.weight.unwrap_or(0.0) * 100.0))
                );
                if status.is_penalized {
                    println!();
                    print_warning("Account is currently penalized due to negative balance");
                    println!("  Submit valid issues to restore your balance >= 0");
                }
            } else {
                print_warning("Miner not registered.");
                println!();
                println!("This could mean:");
                println!("  - You haven't registered your GitHub account yet");
                println!("  - You haven't claimed any valid bounties yet");
                println!();
                println!("To register, run:");
                println!("  bounty");
            }
        }
        Err(e) => {
            print_error(&format!("Failed to fetch status: {}", e));
        }
    }

    Ok(())
}
