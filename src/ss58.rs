use alloc::string::{String, ToString};
use alloc::vec::Vec;
use blake2::{Blake2b512, Digest};

const SS58_PREFIX: &[u8] = b"SS58PRE";
const SUBSTRATE_PREFIX: u16 = 42; // Generic Substrate

/// Convert hex hotkey to SS58 format
pub fn hex_to_ss58(hex: &str) -> Option<String> {
    let hex_clean = hex.strip_prefix("0x").unwrap_or(hex);
    if hex_clean.len() != 64 {
        return None;
    }

    let mut pubkey = [0u8; 32];
    for (i, chunk) in hex_clean.as_bytes().chunks(2).enumerate() {
        if i >= 32 {
            break;
        }
        let s = core::str::from_utf8(chunk).ok()?;
        pubkey[i] = u8::from_str_radix(s, 16).ok()?;
    }

    Some(encode_ss58(&pubkey, SUBSTRATE_PREFIX))
}

/// Convert SS58 to hex format
pub fn ss58_to_hex(ss58: &str) -> Option<String> {
    let pubkey = decode_ss58(ss58)?;
    let mut hex = String::with_capacity(64);
    for byte in pubkey {
        use core::fmt::Write;
        let _ = write!(hex, "{:02x}", byte);
    }
    Some(hex)
}

/// Check if string is SS58 format (starts with 5 for Substrate)
pub fn is_ss58(s: &str) -> bool {
    s.starts_with('5') && s.len() >= 47 && s.len() <= 48
}

/// Check if string is hex format (64 hex chars)
pub fn is_hex(s: &str) -> bool {
    let s = s.strip_prefix("0x").unwrap_or(s);
    s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// Normalize hotkey to SS58 format
pub fn normalize_hotkey(hotkey: &str) -> Option<String> {
    if is_ss58(hotkey) {
        // Validate it's a real SS58
        if decode_ss58(hotkey).is_some() {
            return Some(hotkey.to_string());
        }
        return None;
    }
    if is_hex(hotkey) {
        return hex_to_ss58(hotkey);
    }
    None
}

/// Get canonical storage key for a hotkey (always SS58)
pub fn storage_key(hotkey: &str) -> Option<String> {
    normalize_hotkey(hotkey)
}

fn encode_ss58(pubkey: &[u8; 32], prefix: u16) -> String {
    let mut data = Vec::with_capacity(35);

    if prefix < 64 {
        data.push(prefix as u8);
    } else {
        data.push(((prefix & 0x00FC) >> 2) as u8 | 0x40);
        data.push(((prefix >> 8) as u8) | ((prefix & 0x0003) << 6) as u8);
    }

    data.extend_from_slice(pubkey);

    let checksum = ss58_checksum(&data);
    data.extend_from_slice(&checksum[..2]);

    bs58::encode(data).into_string()
}

fn decode_ss58(ss58: &str) -> Option<[u8; 32]> {
    let data = bs58::decode(ss58).into_vec().ok()?;

    if data.len() < 35 {
        return None;
    }

    let (prefix_len, _prefix) = if data[0] & 0x40 != 0 {
        (
            2,
            ((data[0] as u16 & 0x3F) << 2)
                | ((data[1] as u16) >> 6)
                | ((data[1] as u16 & 0x3F) << 8),
        )
    } else {
        (1, data[0] as u16)
    };

    let pubkey_start = prefix_len;
    let pubkey_end = data.len() - 2;

    if pubkey_end - pubkey_start != 32 {
        return None;
    }

    // Verify checksum
    let checksum = ss58_checksum(&data[..pubkey_end]);
    if checksum[0] != data[pubkey_end] || checksum[1] != data[pubkey_end + 1] {
        return None;
    }

    let mut pubkey = [0u8; 32];
    pubkey.copy_from_slice(&data[pubkey_start..pubkey_end]);
    Some(pubkey)
}

fn ss58_checksum(data: &[u8]) -> [u8; 64] {
    let mut hasher = Blake2b512::new();
    hasher.update(SS58_PREFIX);
    hasher.update(data);
    let result = hasher.finalize();
    let mut checksum = [0u8; 64];
    checksum.copy_from_slice(&result);
    checksum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_ss58() {
        let hex = "da220409678df5f06074a671abdc1f19bc2ba151729fdb9a8e4be284e60c9401";
        let ss58 = hex_to_ss58(hex).unwrap();
        assert!(ss58.starts_with('5'));

        // Round trip
        let back = ss58_to_hex(&ss58).unwrap();
        assert_eq!(back, hex);
    }

    #[test]
    fn test_normalize() {
        let hex = "da220409678df5f06074a671abdc1f19bc2ba151729fdb9a8e4be284e60c9401";
        let ss58 = normalize_hotkey(hex).unwrap();

        // Normalizing SS58 should return same
        let ss58_2 = normalize_hotkey(&ss58).unwrap();
        assert_eq!(ss58, ss58_2);
    }
}
