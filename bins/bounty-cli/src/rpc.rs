use anyhow::{Context, Result};
use serde_json::Value;

const CHALLENGE_ID: &str = "bounty-challenge";

pub async fn rpc_call(
    rpc_url: &str,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> Result<Value> {
    let client = reqwest::Client::new();

    let mut params = serde_json::json!({
        "challengeId": CHALLENGE_ID,
        "method": method,
        "path": path,
    });

    if let Some(b) = body {
        params["body"] = b;
    }

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "challenge_call",
        "params": params,
        "id": 1,
    });

    let response = client
        .post(format!("{}/rpc", rpc_url))
        .json(&request)
        .send()
        .await
        .context("Failed to connect to validator RPC")?;

    let status = response.status();
    let json: Value = response
        .json()
        .await
        .context("Failed to parse RPC response")?;

    if let Some(error) = json.get("error") {
        let msg = error
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown RPC error");
        anyhow::bail!("RPC error (HTTP {}): {}", status, msg);
    }

    let result = json.get("result").cloned().unwrap_or(Value::Null);
    Ok(result)
}
