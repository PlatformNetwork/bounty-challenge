//! Validate command - run as validator

use crate::style::*;
use anyhow::Result;

pub async fn run(platform_url: &str, hotkey: Option<String>) -> Result<()> {
    print_header("Validator Mode");

    println!("Platform:  {}", platform_url);
    if let Some(ref hk) = hotkey {
        println!("Hotkey:    {}", crate::style::truncate_hotkey(hk));
    }
    println!();

    print_info("Validator mode connects to Platform Server and participates in consensus.");
    println!();

    let client = reqwest::Client::new();

    print_info("Checking Platform Server connectivity...");

    match client.get(format!("{}/health", platform_url)).send().await {
        Ok(response) if response.status().is_success() => {
            print_success("Platform Server is reachable");
        }
        Ok(response) => {
            print_error(&format!("Platform Server returned: {}", response.status()));
        }
        Err(e) => {
            print_error(&format!("Failed to connect: {}", e));
        }
    }

    // Check metagraph registration
    if let Some(hotkey) = hotkey {
        print_info("Checking metagraph registration...");

        let metagraph = bounty_challenge::metagraph::MetagraphCache::new(platform_url.to_string());

        match metagraph.refresh().await {
            Ok(count) => {
                print_success(&format!("Loaded {} miners from metagraph", count));

                if metagraph.is_registered(&hotkey) {
                    print_success("Your hotkey is registered on the subnet");
                } else {
                    print_warning("Your hotkey is NOT registered on the subnet");
                    println!("  You need to register on the Bittensor subnet first.");
                }
            }
            Err(e) => {
                print_warning(&format!("Could not load metagraph: {}", e));
            }
        }
    }

    println!();
    print_info("Validator mode is not fully implemented yet.");
    print_info("Use 'bounty server' to run the challenge server instead.");

    Ok(())
}
