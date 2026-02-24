use anyhow::Result;
use console::style;
use serde_json::Value;

use crate::rpc::rpc_call;

fn derive_status(issue: &Value) -> &'static str {
    let has_valid = issue
        .get("has_valid_label")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let has_invalid = issue
        .get("has_invalid_label")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let has_duplicate = issue
        .get("has_duplicate_label")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if has_duplicate {
        "duplicate"
    } else if has_invalid {
        "invalid"
    } else if has_valid {
        "valid"
    } else {
        "pending"
    }
}

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
        "  {:<6} {:<8} {:<30} {:<12} {:<15}",
        style("#").yellow(),
        style("Issue").yellow(),
        style("Repo").yellow(),
        style("Status").yellow(),
        style("Author").yellow(),
    );
    println!("  {}", style("─".repeat(75)).dim());

    for (i, issue) in arr.iter().enumerate() {
        let issue_num = issue
            .get("issue_number")
            .and_then(|v| v.as_u64())
            .map(|n| format!("#{}", n))
            .unwrap_or_else(|| "?".to_string());

        let repo = issue
            .get("repo_name")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let owner = issue
            .get("repo_owner")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let repo_display = format!("{}/{}", owner, repo);
        let repo_short = if repo_display.len() > 28 {
            format!("{}...", &repo_display[..25])
        } else {
            repo_display
        };

        let status = derive_status(issue);
        let author = issue.get("author").and_then(|v| v.as_str()).unwrap_or("?");

        let status_styled = match status {
            "valid" => style(status).green(),
            "pending" => style(status).yellow(),
            "invalid" => style(status).red(),
            "duplicate" => style(status).magenta(),
            _ => style(status).dim(),
        };

        println!(
            "  {:<6} {:<8} {:<30} {:<12} {:<15}",
            i + 1,
            issue_num,
            repo_short,
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
