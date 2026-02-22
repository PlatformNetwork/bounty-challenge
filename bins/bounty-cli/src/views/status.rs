use anyhow::Result;
use console::style;
use dialoguer::Input;

use crate::rpc::rpc_call;

pub async fn run(rpc_url: &str) -> Result<()> {
    println!("\n{}", style("Miner Status").cyan().bold());
    println!("{}\n", style("─".repeat(40)).dim());

    let hotkey: String = Input::new().with_prompt("SS58 hotkey").interact_text()?;

    let path = format!("/status/{}", hotkey.trim());
    let result = rpc_call(rpc_url, "GET", &path, None).await?;
    let body = result.get("body").unwrap_or(&result);

    let registered = body
        .get("registered")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !registered {
        println!("\n{} {}", style("Hotkey").dim(), style(&hotkey).yellow());
        println!(
            "{}",
            style("Not registered. Use Register to sign up.").red()
        );
        println!();
        return Ok(());
    }

    let github = body
        .get("github_username")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let valid = body
        .get("valid_issues_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let invalid = body
        .get("invalid_issues_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let weight = body.get("weight").and_then(|v| v.as_f64()).unwrap_or(0.0);

    let balance = body.get("balance");
    let duplicates = balance
        .and_then(|b| b.get("duplicate_count"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let stars = balance
        .and_then(|b| b.get("star_count"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let penalized = balance
        .and_then(|b| b.get("is_penalized"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    println!();
    println!("  {} {}", style("Hotkey:").dim(), style(&hotkey).green());
    println!("  {} {}", style("GitHub:").dim(), style(github).cyan());
    println!();
    println!(
        "  {} {}",
        style("Valid Issues:").dim(),
        style(valid).green()
    );
    println!(
        "  {} {}",
        style("Invalid Issues:").dim(),
        style(invalid).red()
    );
    println!(
        "  {} {}",
        style("Duplicates:").dim(),
        style(duplicates).yellow()
    );
    println!("  {} {}", style("Stars:").dim(), style(stars).yellow());
    println!();
    println!(
        "  {} {}",
        style("Weight:").dim(),
        style(format!("{:.4}", weight)).bold()
    );

    if penalized {
        println!(
            "  {} {}",
            style("Status:").dim(),
            style("PENALIZED (weight = 0)").red().bold()
        );
    } else {
        println!(
            "  {} {}",
            style("Status:").dim(),
            style("Active").green().bold()
        );
    }

    println!();
    Ok(())
}
