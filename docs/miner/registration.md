# Registration Guide

This guide explains how to register your GitHub username with your Bittensor hotkey on the Bounty Challenge.

## Overview

Registration links your on-chain Bittensor hotkey to your GitHub username. This is required before you can earn rewards for submitting valid issues.

## Registration via Interactive CLI (Recommended)

The easiest way to register is using the interactive CLI:

```bash
bounty-cli
```

1. Select **Register** from the menu
2. Enter your GitHub username
3. Enter your 24-word mnemonic (input is hidden)
4. The CLI will automatically sign and submit the registration

The CLI uses platform-v2's authentication format with headers (`X-Hotkey`, `X-Signature`, `X-Nonce`).

## Authentication Format (platform-v2)

All authenticated requests use sr25519 signatures with the following format:

**Signed message:**
```
challenge:bounty-challenge:{METHOD}:{PATH}:{BODY_HASH}:{NONCE}
```

**Headers:**
- `X-Hotkey`: Your hotkey public key (hex, 64 chars)
- `X-Signature`: sr25519 signature (hex, 128 chars)  
- `X-Nonce`: `{unix_timestamp}:{random_hex}`

**Body hash:** SHA256 of the JSON request body (hex encoded)

## Verify Registration

After registering, select **My Status** from the CLI menu and enter your hotkey to verify.

## Direct API Registration (Advanced)

You can register directly via JSON-RPC with the new authentication format:

```bash
# Generate the auth values:
# - BODY_HASH: SHA256 of '{"github_username":"JohnDoe"}' (hex)
# - NONCE: {timestamp}:{random_hex}
# - MESSAGE: challenge:bounty-challenge:POST:/register:{BODY_HASH}:{NONCE}
# - SIGNATURE: sr25519 sign(MESSAGE)

curl -X POST https://chain.platform.network/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "challenge_call",
    "params": {
      "challengeId": "bounty-challenge",
      "method": "POST",
      "path": "/register",
      "body": {
        "github_username": "JohnDoe"
      },
      "headers": {
        "X-Hotkey": "your_hotkey_hex_64_chars",
        "X-Signature": "your_signature_hex_128_chars",
        "X-Nonce": "1705590000:abc123def456"
      }
    },
    "id": 1
  }'
```

> **Note**: The CLI handles all the signing automatically. Direct API usage is only recommended for advanced integrations.

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "Registration failed" | Check that the signature is correct and timestamp is recent |
| "Unauthorized" | Ensure the hotkey is registered on the subnet |
| "Invalid signature" | Make sure you signed with sr25519 and the message format is exact |
| "Timestamp expired" | The timestamp must be within 5 minutes of the validator's server time |

## Important Notes

- Each hotkey can only be registered to **one** GitHub username
- Each GitHub username can only be linked to **one** hotkey
- Registration is permanent — you cannot change your linked GitHub username
- The signature proves ownership of the hotkey without revealing your private key
