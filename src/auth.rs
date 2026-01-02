//! Authentication and Authorization
//!
//! - SS58 hotkey validation
//! - Sr25519 signature verification  
//! - GitHub OAuth support

use sp_core::crypto::Ss58Codec;
use sp_core::sr25519::{Public, Signature};
use tracing::debug;

/// Check if a string is a valid SS58-encoded sr25519 public key
pub fn is_valid_ss58_hotkey(hotkey: &str) -> bool {
    if hotkey.len() < 40 || hotkey.len() > 60 {
        return false;
    }
    Public::from_ss58check(hotkey).is_ok()
}

/// Verify an sr25519 signature
pub fn verify_signature(hotkey: &str, message: &str, signature_hex: &str) -> bool {
    let public_key = match Public::from_ss58check(hotkey) {
        Ok(pk) => pk,
        Err(e) => {
            debug!("Failed to parse SS58 hotkey: {}", e);
            return false;
        }
    };

    let sig_hex = signature_hex
        .strip_prefix("0x")
        .unwrap_or(signature_hex)
        .to_lowercase();

    let sig_bytes = match hex::decode(&sig_hex) {
        Ok(b) => b,
        Err(e) => {
            debug!("Failed to decode signature hex: {}", e);
            return false;
        }
    };

    if sig_bytes.len() != 64 {
        debug!("Invalid signature length: {} (expected 64)", sig_bytes.len());
        return false;
    }

    let mut sig_array = [0u8; 64];
    sig_array.copy_from_slice(&sig_bytes);
    let signature = Signature::from_raw(sig_array);

    use sp_core::Pair;
    sp_core::sr25519::Pair::verify(&signature, message.as_bytes(), &public_key)
}

/// Create message for registration
pub fn create_register_message(github_username: &str, timestamp: i64) -> String {
    format!("register_github:{}:{}", github_username, timestamp)
}

/// Create message for claiming bounties
pub fn create_claim_message(issue_numbers: &[u32], timestamp: i64) -> String {
    let issues_str = issue_numbers
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!("claim_bounties:{}:{}", issues_str, timestamp)
}

/// Check if timestamp is within acceptable window (5 minutes)
pub fn is_timestamp_valid(timestamp: i64) -> bool {
    let now = chrono::Utc::now().timestamp();
    let window = 5 * 60;
    (now - timestamp).abs() < window
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ss58_validation() {
        assert!(is_valid_ss58_hotkey(
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        ));
        assert!(!is_valid_ss58_hotkey("not_a_valid_address"));
        assert!(!is_valid_ss58_hotkey(""));
    }

    #[test]
    fn test_timestamp_validation() {
        let now = chrono::Utc::now().timestamp();
        assert!(is_timestamp_valid(now));
        assert!(is_timestamp_valid(now - 60));
        assert!(!is_timestamp_valid(now - 600));
    }
}
