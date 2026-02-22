use anyhow::Result;
use console::style;
use serde_json::Value;

use crate::rpc::rpc_call;

fn print_issues(data: &Value) {
    let body = data.get("body").unwrap_or(data);
    let arr = match body.as_array() {
        Some(a) => a,
        None => {
            println!("  {}", style("No issues found.").dim());
            return;
        }
    };

    if arr.is_empty() {
        println!("  {}", style("No issues found.").dim());
        return;
    }

    println!(
        "  {:<6} {:<50} {:<12} {:<15}",
        style("#").yellow(),
        style("URL / Title").yellow(),
        style("Status").yellow(),
        style("Author").yellow(),
    );
    println!("  {}", style("─".repeat(85)).dim());

    for (i, issue) in arr.iter().enumerate() {
        let url = issue
            .get("url")
            .or_else(|| issue.get("issue_url"))
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let title = issue
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or(url);
        let display = if title.len() > 47 {
            format!("{}...", &title[..47])
        } else {
            title.to_string()
        };
        let status = issue
            .get("status")
            .or_else(|| issue.get("state"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let author = issue
            .get("author")
            .or_else(|| issue.get("github_username"))
            .and_then(|v| v.as_str())
            .unwrap_or("?");

        let status_styled = match status {
            "valid" | "closed" => style(status).green(),
            "pending" | "open" => style(status).yellow(),
            "invalid" => style(status).red(),
            _ => style(status).dim(),
        };

        println!(
            "  {:<6} {:<50} {:<12} {:<15}",
            i + 1,
            display,
            status_styled,
            author,
        );
    }
}

pub async fn run_all(rpc_url: &str) -> Result<()> {
    println!("\n{}", style("All Issues").cyan().bold());
    println!("{}\n", style("─".repeat(40)).dim());

    let result = rpc_call(rpc_url, "GET", "/issues", None).await?;
    print_issues(&result);
    println!();
    Ok(())
}

pub async fn run_pending(rpc_url: &str) -> Result<()> {
    println!("\n{}", style("Pending Issues").cyan().bold());
    println!("{}\n", style("─".repeat(40)).dim());

    let result = rpc_call(rpc_url, "GET", "/issues/pending", None).await?;
    print_issues(&result);
    println!();
    Ok(())
}
