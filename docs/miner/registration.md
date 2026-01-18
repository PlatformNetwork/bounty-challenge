# Registration Guide

This guide explains how to register your GitHub account with your miner hotkey.

## Overview

Registration links your GitHub username to your Bittensor hotkey. This allows the system to:
1. Verify that issues you create belong to you
2. Credit rewards to your hotkey
3. Prevent impersonation

## Registration Methods

### Method 1: Interactive Wizard (Recommended)

```bash
bounty
```

The wizard guides you through:
1. Entering your secret key
2. Verifying your hotkey
3. Entering your GitHub username
4. Signing and submitting

### Method 2: With Environment Variables

```bash
# Set your secret key
export MINER_SECRET_KEY="your-64-char-hex-or-mnemonic"

# Run wizard (will auto-detect key)
bounty wizard
```

## Secret Key Formats

The CLI accepts several key formats:

### 64-Character Hex Seed

```
a1b2c3d4e5f6789012345678901234567890123456789012345678901234abcd
```

This is a 32-byte seed encoded as hexadecimal.

### 12+ Word Mnemonic

```
abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
```

Standard BIP-39 mnemonic phrase.

### SURI Format (Testing Only)

```
//Alice
//Bob
```

Substrate URI format, useful for testing with well-known keys.

## Signature Verification

### How It Works

1. **Message Creation**: `register_github:{username}:{timestamp}`
2. **Signing**: sr25519 signature using your secret key
3. **Verification**: Server verifies signature matches the claimed hotkey

### Security

- **Timestamp**: Must be within 5 minutes (prevents replay attacks)
- **Username**: Lowercase for consistency
- **Hotkey**: Derived from your signature, not trusted from input

## Changing Registration

### Change GitHub Username

To link a different GitHub username:
1. Run the wizard again with the same hotkey
2. Enter the new GitHub username
3. The old link is replaced

### Change Hotkey

To link your GitHub to a different hotkey:
1. Contact support (username can only link to one hotkey)
2. Or create a new GitHub account

## Troubleshooting

### "Invalid signature" Error

- **Cause**: Signature doesn't match the claimed hotkey
- **Fix**: Ensure you're using the correct secret key

### "Timestamp expired" Error

- **Cause**: Request took too long or system clock is wrong
- **Fix**: Check your system clock and try again

### "Username already registered" Error

- **Cause**: This GitHub username is linked to another hotkey
- **Fix**: Use a different GitHub account or contact support

### "Hotkey already registered" Error

- **Cause**: This hotkey is linked to another GitHub username
- **Fix**: Run wizard again to update the linked username

## API Registration

For programmatic registration:

```bash
curl -X POST https://chain.platform.network/api/v1/bridge/bounty-challenge/register \
  -H "Content-Type: application/json" \
  -d '{
    "hotkey": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    "github_username": "johndoe",
    "signature": "0x...",
    "timestamp": 1705590000
  }'
```

See the [API Reference](../reference/api-reference.md) for details.
