use anyhow::Result;
use console::style;

use crate::rpc::rpc_call;

pub async fn run_sync(rpc_url: &str) -> Result<()> {
    println!("\n{}", style("Sync Consensus").cyan().bold());
    println!("{}\n", style("─".repeat(40)).dim());

    let result = rpc_call(rpc_url, "GET", "/sync/consensus", None).await?;
    let body = result.get("body").unwrap_or(&result);

    let pretty = serde_json::to_string_pretty(body).unwrap_or_else(|_| body.to_string());
    println!("{}", pretty);
    println!();
    Ok(())
}

pub async fn run_issue(rpc_url: &str) -> Result<()> {
    println!("\n{}", style("Issue Consensus").cyan().bold());
    println!("{}\n", style("─".repeat(40)).dim());

    let result = rpc_call(rpc_url, "POST", "/issue/consensus", None).await?;
    let body = result.get("body").unwrap_or(&result);

    let pretty = serde_json::to_string_pretty(body).unwrap_or_else(|_| body.to_string());
    println!("{}", pretty);
    println!();
    Ok(())
}
