use anyhow::Result;
use console::style;
use dialoguer::Select;

use crate::rpc::rpc_call;

pub async fn run(rpc_url: &str) -> Result<()> {
    println!("\n{}", style("Timeout Configuration").cyan().bold());
    println!("{}\n", style("─".repeat(40)).dim());

    let choices = &["View current config", "Back"];
    let selection = Select::new()
        .with_prompt("Action")
        .items(choices)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            let result = rpc_call(rpc_url, "GET", "/config/timeout", None).await?;
            let body = result.get("body").unwrap_or(&result);
            let pretty = serde_json::to_string_pretty(body).unwrap_or_else(|_| body.to_string());
            println!("\n{}", pretty);
        }
        _ => {}
    }

    println!();
    Ok(())
}
