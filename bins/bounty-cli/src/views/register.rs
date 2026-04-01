use anyhow::Result;
use console::style;
use dialoguer::{Input, Password};
use sp_core::{crypto::Pair as PairTrait, sr25519::Pair};
use bip39::{Language, Mnemonic};

use crate::rpc::rpc_call_auth;

pub async fn run(rpc_url: &str) -> Result<()> {
    println!("\n{}", style("Register GitHub Username").cyan().bold());
    println!("{}\n", style("─".repeat(40)).dim());

    let github: String = Input::new()
        .with_prompt("GitHub username")
        .interact_text()?;

    let mnemonic: String = Password::new()
        .with_prompt("Enter your 12 or 24-word mnemonic (hidden)")
        .interact()?;

    let mnemonic = mnemonic.trim();
    let words: Vec<&str> = mnemonic.split_whitespace().collect();
    if words.len() != 12 && words.len() != 24 {
        anyhow::bail!(
            "Expected 12 or 24 words, got {}. Your mnemonic should have exactly 12 or 24 words separated by spaces.",
            words.len()
        );
    }

    println!("{}", style("Deriving sr25519 keypair...").dim());

    let pair = match Pair::from_phrase(mnemonic, None) {
        Ok((pair, _seed)) => pair,
        Err(e) => {
            let error_detail = match Mnemonic::parse_in(Language::English, mnemonic) {
                Ok(_) => format!("{}", e),
                Err(bip39_err) => {
                    let error_msg = format!("{}", bip39_err);
                    if error_msg.contains("Invalid word") || error_msg.contains("unknown") {
                        let wordlist = Language::English.word_list();
                        let wordlist_set: std::collections::HashSet<&str> = 
                            wordlist.iter().copied().collect();
                        let invalid_words: Vec<&str> = words.iter()
                            .filter(|w| !wordlist_set.contains(w.to_lowercase().as_str()))
                            .copied()
                            .collect();
                        
                        if !invalid_words.is_empty() {
                            let words_str = invalid_words.join("', '");
                            format!("Unknown word(s) not in BIP39 wordlist: '{}'. \
                                    Check for typos. All words must be lowercase.", words_str)
                        } else {
                            error_msg
                        }
                    } else if error_msg.contains("checksum") {
                        "Invalid checksum. The last word may be incorrect or the mnemonic may be corrupted.".to_string()
                    } else {
                        error_msg
                    }
                }
            };
            anyhow::bail!("Invalid mnemonic: {}", error_detail);
        }
    };

    let hotkey_ss58 = sp_core::crypto::Ss58Codec::to_ss58check(&pair.public());

    println!(
        "  {} {}",
        style("Hotkey:").dim(),
        style(&hotkey_ss58).green()
    );

    let body = serde_json::json!({
        "github_username": github,
    });

    println!("{}", style("Sending authenticated registration...").dim());

    let result = rpc_call_auth(rpc_url, "POST", "/register", Some(body), &pair).await?;
    let response_body = result.get("body").unwrap_or(&result);

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
