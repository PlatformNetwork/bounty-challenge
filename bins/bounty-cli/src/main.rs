mod rpc;
mod tui;
mod views;

use anyhow::Result;
use console::style;
use dialoguer::{Input, Select};
use structopt::StructOpt;

const DEFAULT_RPC_URL: &str = "https://chain.platform.network";

const MENU_ITEMS: &[&str] = &[
    "Leaderboard        (live dashboard)",
    "Challenge Stats    (live dashboard)",
    "Weights            (live dashboard)",
    "My Status",
    "Issues",
    "Pending Issues",
    "Register",
    "Claim Bounty",
    "Change RPC URL",
    "Quit",
    "Cortex Feedback",
];

#[derive(StructOpt)]
struct CortexFeedback {
    #[structopt(short = "s", long = "session")]
    session: Option<String>,
    #[structopt(short = "b", long = "bug")]
    bug: Option<String>,
    #[structopt(short = "m", long = "message")]
    message: String,
}

fn print_header(rpc_url: &str) {
    println!();
    println!("  {}", style("bounty-challenge").cyan().bold());
    println!("  {} {}", style("RPC:").dim(), style(rpc_url).green());
    println!();
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut rpc_url =
        std::env::var("BOUNTY_RPC_URL").unwrap_or_else(|_| DEFAULT_RPC_URL.to_string());

    loop {
        print_header(&rpc_url);

        let selection = Select::new()
            .with_prompt("Select an action")
            .items(MENU_ITEMS)
            .default(0)
            .interact_opt()?;

        let selection = match selection {
            Some(s) => s,
            None => break,
        };

        let result = match selection {
            0 => tui::leaderboard::run(&rpc_url).await,
            1 => tui::stats::run(&rpc_url).await,
            2 => tui::weights::run(&rpc_url).await,
            3 => views::status::run(&rpc_url).await,
            4 => views::issues::run_all(&rpc_url).await,
            5 => views::issues::run_pending(&rpc_url).await,
            6 => views::register::run(&rpc_url).await,
            7 => views::claim::run(&rpc_url).await,
            8 => {
                let new_url: String = Input::new()
                    .with_prompt("New RPC URL")
                    .default(rpc_url.clone())
                    .interact_text()?;
                rpc_url = new_url.trim_end_matches('/').to_string();
                println!(
                    "  {} {}",
                    style("RPC updated:").dim(),
                    style(&rpc_url).green()
                );
                Ok(())
            }
            9 => {
                let feedback = CortexFeedback::from_args();
                let mut file_path = std::path::PathBuf::from("~/.cortex/feedback");
                file_path.push(format!("{}.json", chrono::Utc::now().timestamp()));
                let mut file = std::fs::File::create(file_path)?;
                let json = serde_json::json!({
                    "session_id": feedback.session.unwrap_or_default(),
                    "message": feedback.message,
                });
                serde_json::to_writer_pretty(&mut file, &json)?;
                Ok(())
            }
            10 => break,
            _ => break,
        };

        if let Err(e) = result {
            println!("\n  {} {}\n", style("Error:").red().bold(), e);
        }

        println!("{}", style("Press Enter to continue...").dim());
        let mut buf = String::new();
        let _ = std::io::stdin().read_line(&mut buf);
    }

    println!("{}", style("Goodbye!").dim());
    Ok(())
}