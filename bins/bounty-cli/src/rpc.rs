use anyhow::{Context, Result};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sp_core::{crypto::Pair as PairTrait, sr25519::Pair};
use std::collections::HashMap;

const CHALLENGE_ID: &str = "bounty-challenge";

fn canonicalize_json(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let mut pairs: Vec<_> = map.iter().collect();
            pairs.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
            let inner: Vec<String> = pairs
                .iter()
                .map(|(k, v)| {
                    format!(
                        "{}:{}",
                        serde_json::to_string(k).unwrap_or_default(),
                        canonicalize_json(v)
                    )
                })
                .collect();
            format!("{{{}}}", inner.join(","))
        }
        Value::Array(arr) => {
            let inner: Vec<String> = arr.iter().map(canonicalize_json).collect();
            format!("[{}]", inner.join(","))
        }
        _ => serde_json::to_string(value).unwrap_or_else(|_| "null".to_string()),
    }
}

/// RPC call without authentication
pub async fn rpc_call(
    rpc_url: &str,
    method: &str,
    path: &str,
    body: Option<Value>,
) -> Result<Value> {
    rpc_call_internal(rpc_url, method, path, body, None).await
}

/// RPC call with sr25519 authentication
///
/// Signs the request using the new platform-v2 format:
/// - Message: `challenge:{challenge_id}:{method}:{path}:{body_hash}:{nonce}`
/// - Headers: `X-Hotkey`, `X-Signature`, `X-Nonce`
pub async fn rpc_call_auth(
    rpc_url: &str,
    method: &str,
    path: &str,
    body: Option<Value>,
    keypair: &Pair,
) -> Result<Value> {
    let body_bytes = body
        .as_ref()
        .map(|b| canonicalize_json(b).into_bytes())
        .unwrap_or_default();

    // Create nonce: {timestamp}:{random}
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    let random: u64 = rand::random();
    let nonce = format!("{}:{:016x}", timestamp, random);

    // Hash the canonicalized body
    let body_hash = hex::encode(Sha256::digest(&body_bytes));

    // Create the signed message
    let message = format!(
        "challenge:{}:{}:{}:{}:{}",
        CHALLENGE_ID, method, path, body_hash, nonce
    );

    // Sign
    let signature = keypair.sign(message.as_bytes());
    let sig_hex = hex::encode(signature.0);

    // Get hotkey as hex (not SS58)
    let hotkey_hex = hex::encode(keypair.public().0);

    let mut headers = HashMap::new();
    headers.insert("X-Hotkey".to_string(), hotkey_hex);
    headers.insert("X-Signature".to_string(), sig_hex);
    headers.insert("X-Nonce".to_string(), nonce);

    rpc_call_internal(rpc_url, method, path, body, Some(headers)).await
}

async fn rpc_call_internal(
    rpc_url: &str,
    method: &str,
    path: &str,
    body: Option<Value>,
    headers: Option<HashMap<String, String>>,
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

    if let Some(h) = headers {
        params["headers"] = serde_json::to_value(h)?;
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
