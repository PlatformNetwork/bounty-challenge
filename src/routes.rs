let mut is_authenticated = true;
let mut rpc_url =
    std::env::var("BOUNTY_RPC_URL").unwrap_or_else(|_| DEFAULT_RPC_URL.to_string());

loop {
    print_header(&rpc_url);
    if !is_authenticated {
        println!("  {}", style("Session expired or authentication required").red().bold());
        let selection = Select::new()
            .with_prompt("Select an action")
            .items(&["Login with another account", "Cancel"])
            .default(0)
            .interact_opt()?;
        match selection {
            Some(0) => {
                // Login with another account logic
                is_authenticated = true;
            }
            Some(1) => {
                // Cancel logic
            }
            None => {}
        }
    } else {
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
            9 => break,
            _ => break,
        };
        if let Err(e) = result {
            println!("\n  {} {}\n", style("Error:").red().bold(), e);
        }
        println!("{}", style("Press Enter to continue...").dim());
        let mut buf = String::new();
        let _ = std::io::stdin().read_line(&mut buf);
    }
}