# Registration Guide

This guide explains how to register your GitHub username with your Bittensor hotkey on the Bounty Challenge.

## Overview

Registration links your on-chain Bittensor hotkey to your GitHub username. This is required before you can earn rewards for submitting valid issues.

## Registration Steps

### 1. Prepare Your Signature

The registration requires an sr25519 signature proving you own the hotkey. Sign the following message:

```
register_github:{github_username_lowercase}:{unix_timestamp}
```

For example, if your GitHub username is `JohnDoe` and the current timestamp is `1705590000`:

```
register_github:johndoe:1705590000
```

> **Note**: The username in the message must be **lowercase**, regardless of your actual GitHub username casing.

### 2. Register via CLI

```bash
./target/release/bounty-cli register \
  --hotkey 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY \
  --github JohnDoe \
  --signature 0xabc123...def456 \
  --timestamp 1705590000 \
  --rpc-url http://VALIDATOR_IP:8080
```

### 3. Verify Registration

```bash
./target/release/bounty-cli status \
  --hotkey 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY \
  --rpc-url http://VALIDATOR_IP:8080
```

## Generating the Signature

### Python (using substrate-interface)

```python
import time
from substrateinterface import Keypair

keypair = Keypair.create_from_mnemonic("your mnemonic here")

timestamp = int(time.time())
message = f"register_github:johndoe:{timestamp}"
signature = keypair.sign(message.encode()).hex()

print(f"Hotkey:    {keypair.ss58_address}")
print(f"Signature: 0x{signature}")
print(f"Timestamp: {timestamp}")
```

### JavaScript (using @polkadot/keyring)

```javascript
const { Keyring } = require('@polkadot/keyring');
const { u8aToHex } = require('@polkadot/util');

const keyring = new Keyring({ type: 'sr25519' });
const pair = keyring.addFromMnemonic('your mnemonic here');

const timestamp = Math.floor(Date.now() / 1000);
const message = `register_github:johndoe:${timestamp}`;
const signature = pair.sign(message);

console.log(`Hotkey:    ${pair.address}`);
console.log(`Signature: ${u8aToHex(signature)}`);
console.log(`Timestamp: ${timestamp}`);
```

## Direct API Registration

You can also register directly via the chain RPC without the CLI:

### Using curl (HTTP REST)

```bash
curl -X POST http://VALIDATOR_IP:8080/challenge/bounty-challenge/register \
  -H "Content-Type: application/json" \
  -d '{
    "hotkey": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    "github_username": "JohnDoe",
    "signature": "0xabc123...def456",
    "timestamp": 1705590000
  }'
```

### Using JSON-RPC

```bash
curl -X POST http://VALIDATOR_IP:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "challenge_call",
    "params": {
      "challengeId": "bounty-challenge",
      "method": "POST",
      "path": "/register",
      "body": {
        "hotkey": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "github_username": "JohnDoe",
        "signature": "0xabc123...def456",
        "timestamp": 1705590000
      }
    },
    "id": 1
  }'
```

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
