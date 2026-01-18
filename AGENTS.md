# Bounty Challenge - Miner Guide

Complete documentation for miners participating in the Bounty Challenge.

## Table of Contents

1. [Overview](#overview)
2. [Registration Flow](#registration-flow)
3. [Creating Valid Issues](#creating-valid-issues)
4. [Reward System](#reward-system)
5. [CLI Reference](#cli-reference)
6. [API Integration](#api-integration)
7. [Best Practices](#best-practices)

---

## Overview

```mermaid
flowchart TB
    subgraph Platform["Platform Server<br/>https://chain.platform.network"]
        Bridge["Bridge API<br/>/api/v1/bridge/bounty-challenge/"]
        Features["• Miner registration (sr25519)<br/>• Issue tracking<br/>• Weight calculation"]
        Bridge --- Features
    end
    
    Platform --> Target["📋 PlatformNetwork/bounty-challenge<br/>(Issues submitted here)"]
    
    Target -.->|"❌ NOT counted"| Cortex["CortexLM/cortex"]
    Target -.->|"❌ NOT counted"| Fabric["CortexLM/fabric"]
```

### Key Components

| Component | Description |
|-----------|-------------|
| **Platform Server** | Central orchestrator at `chain.platform.network` |
| **Bridge API** | Routes all traffic through `/api/v1/bridge/bounty-challenge/` |
| **Target Repos** | CortexLM/cortex, CortexLM/fabric (configurable) |
| **Valid Label** | Only issues with this label count for rewards |

### Where to Submit Issues

> **IMPORTANT**: All issues must be submitted to the bounty-challenge repository to receive rewards!

| Repository | Status |
|------------|--------|
| [PlatformNetwork/bounty-challenge](https://github.com/PlatformNetwork/bounty-challenge/issues) | ✅ Rewards eligible |
| CortexLM/cortex | ❌ Not counted for rewards |
| CortexLM/fabric | ❌ Not counted for rewards |

---

## Registration Flow

```mermaid
flowchart LR
    subgraph Step1["1. Secret Key"]
        K1["64-char hex<br/>or mnemonic"]
    end
    
    subgraph Step2["2. Derive Hotkey"]
        K2["SS58 format<br/>(5xxx...)"]
    end
    
    subgraph Step3["3. Sign Message"]
        K3["sr25519 signature"]
    end
    
    subgraph Step4["4. Submit"]
        K4["Verified &<br/>stored in DB"]
    end
    
    Step1 --> Step2 --> Step3 --> Step4
```

### Step-by-Step

1. **Run the Wizard**
   ```bash
   bounty
   ```

2. **Enter Your Secret Key**
   
   The CLI accepts:
   - **64-character hex seed**: `a1b2c3d4...` (32 bytes)
   - **12+ word mnemonic**: `word1 word2 word3 ...`
   - **SURI format**: `//Alice` (for testing)

3. **Verify Your Hotkey**
   
   The CLI derives your SS58 hotkey and displays it:
   ```
   ✓ Hotkey: 5GrwvaEF...vQGn
   ```

4. **Enter GitHub Username**
   
   Enter the exact username you use on GitHub (case-insensitive).

5. **Sign and Submit**
   
   The CLI:
   - Creates message: `register_github:{username}:{timestamp}`
   - Signs with sr25519
   - Submits to platform

6. **Confirmation**
   ```
   ✓ Registration successful!
   Your GitHub account @username is now linked.
   ```

### Message Format

```
register_github:{github_username_lowercase}:{unix_timestamp}
```

Example:
```
register_github:johndoe:1705590000
```

### Signature Verification

The server verifies:
1. Timestamp is within 5 minutes (replay protection)
2. Signature matches the claimed hotkey
3. Message format is correct

---

## Creating Valid Issues

> **WARNING**: Issues must be submitted to [PlatformNetwork/bounty-challenge](https://github.com/PlatformNetwork/bounty-challenge/issues) to receive rewards. Issues in other repos will NOT be counted!

### Issue Requirements

For an issue to earn rewards, it must:

1. **Be on the bounty-challenge repository** (this repo!)
2. **Be created by your registered GitHub account**
3. **Be closed by maintainers**
4. **Have the `valid` label** (added by maintainers)

### Valid Issue Types

| Type | Description | Example |
|------|-------------|---------|
| **Bug Report** | Reproducible software defects | "CLI crashes on startup with Python 3.12" |
| **Security Issue** | Vulnerabilities (responsible disclosure) | "XSS in web interface" |
| **Feature Request** | Well-reasoned feature proposals | "Add --quiet flag for CI usage" |
| **Documentation** | Doc errors, gaps, improvements | "Installation docs missing Windows steps" |

### Issue Template

```markdown
## Description
[Clear description of the bug/feature/issue]

## Steps to Reproduce (for bugs)
1. Step one
2. Step two
3. Step three

## Expected Behavior
[What should happen]

## Actual Behavior
[What actually happens]

## Environment
- OS: [e.g., Ubuntu 22.04]
- Cortex version: [e.g., 0.1.0]
- Python version: [if applicable]

## Additional Context
[Screenshots, logs, etc.]
```

### Validation Process

```mermaid
flowchart TD
    A["📝 Issue Created"] --> B["👀 Maintainer Review"]
    B --> C{Valid?}
    C -->|"✅ Yes"| D["Close + 'valid' label"]
    C -->|"❌ No"| E["Close without label"]
    D --> F["💰 Reward credited"]
    E --> G["No reward"]
```

### Label Protection

The `valid` label is protected by GitHub Actions:
- Only authorized maintainers can add/remove it
- Unauthorized changes are automatically reverted
- A comment explains the protection

---

## Reward System

### How Rewards Work

```mermaid
flowchart TD
    subgraph Calc["ADAPTIVE REWARD CALCULATION"]
        A["1️⃣ Count valid issues (24h)"]
        B["2️⃣ Calculate max weight<br/>max = min(total/250, 1.0)"]
        C{"total ≤ 100?"}
        D["weight/issue = 0.01"]
        E["weight/issue = 0.01 × (100/total)"]
        F["4️⃣ User weight<br/>= min(user_issues × weight/issue, max)"]
        
        A --> B --> C
        C -->|"Yes"| D --> F
        C -->|"No"| E --> F
    end
```

### Formulas

**Maximum Daily Weight:**

$$W_{max} = \min\left(\frac{N_{total}}{250}, 1.0\right)$$

**Weight per Issue (Adaptive):**

$$w_{issue} = \begin{cases} 
0.01 & \text{if } N_{total} \leq 100 \\ 
0.01 \times \frac{100}{N_{total}} & \text{if } N_{total} > 100
\end{cases}$$

**User Weight:**

$$W_{user} = \min(n_{user} \times w_{issue}, W_{max})$$

### Example Calculations

**Scenario 1: Low Activity Day (50 issues total)**

| Metric | Value |
|--------|-------|
| Total issues | 50 |
| Max weight | 50/250 = 0.20 |
| Weight/issue | 0.01 |
| Your issues | 5 |
| Your weight | 5 × 0.01 = 0.05 |

**Scenario 2: High Activity Day (300 issues total)**

| Metric | Value |
|--------|-------|
| Total issues | 300 |
| Max weight | 1.0 (capped) |
| Weight/issue | 0.01 × (100/300) = 0.0033 |
| Your issues | 10 |
| Your weight | 10 × 0.0033 = 0.033 |

### Weight Reference Table

| Total Issues/24h | Max Weight | Weight/Issue | 5 Issues | 10 Issues | 20 Issues |
|-----------------|------------|--------------|----------|-----------|-----------|
| 50 | 0.20 | 0.0100 | 0.050 | 0.100 | 0.200 |
| 100 | 0.40 | 0.0100 | 0.050 | 0.100 | 0.200 |
| 150 | 0.60 | 0.0067 | 0.033 | 0.067 | 0.133 |
| 200 | 0.80 | 0.0050 | 0.025 | 0.050 | 0.100 |
| 250 | 1.00 | 0.0040 | 0.020 | 0.040 | 0.080 |
| 500 | 1.00 | 0.0020 | 0.010 | 0.020 | 0.040 |

### 24-Hour Rolling Window

- Rewards are calculated based on the last 24 hours
- As issues age past 24h, they no longer contribute to weights
- This encourages consistent, daily participation

---

## CLI Reference

### Installation

```bash
# Clone and build
git clone https://github.com/PlatformNetwork/bounty-challenge.git
cd bounty-challenge
cargo build --release

# Add to PATH
export PATH="$PWD/target/release:$PATH"
```

### Commands

#### `bounty` / `bounty wizard`

Interactive registration wizard.

```bash
bounty
```

#### `bounty status`

Check your registration status and rewards.

```bash
bounty status --hotkey 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
```

Output:
```
Status for 5GrwvaEF...
  Registered: true
  GitHub: @johndoe
  Issues (24h): 5
  Weight: 0.05
```

#### `bounty leaderboard`

View current standings.

```bash
bounty leaderboard --limit 20
```

Output:
```
Rank  GitHub          Issues  Weight
1     @alice          12      0.12
2     @bob            8       0.08
3     @charlie        5       0.05
...
```

#### `bounty config`

Show challenge configuration.

```bash
bounty config
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PLATFORM_URL` | `https://chain.platform.network` | Platform server URL |
| `MINER_HOTKEY` | - | Your hotkey (for status command) |

---

## API Integration

### Base URL

```
https://chain.platform.network/api/v1/bridge/bounty-challenge/
```

### Register Endpoint

**POST** `/register`

```json
{
  "hotkey": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
  "github_username": "johndoe",
  "signature": "0x...",
  "timestamp": 1705590000
}
```

Response:
```json
{
  "success": true,
  "message": "Successfully registered @johndoe with your hotkey."
}
```

### Status Endpoint

**GET** `/status/{hotkey}`

Response:
```json
{
  "registered": true,
  "github_username": "johndoe",
  "valid_issues_count": 5,
  "weight": 0.05
}
```

### Leaderboard Endpoint

**GET** `/leaderboard?limit=20`

Response:
```json
{
  "leaderboard": [
    {
      "github_username": "alice",
      "hotkey": "5GrwvaEF...",
      "issues_resolved_24h": 12,
      "weight": 0.12
    },
    ...
  ]
}
```

### Stats Endpoint

**GET** `/stats`

Response:
```json
{
  "total_bounties": 150,
  "total_miners": 25,
  "challenge_id": "bounty-challenge",
  "version": "0.1.0"
}
```

---

## Best Practices

### Issue Quality

1. **Be specific** - Provide exact steps to reproduce
2. **Include context** - OS, version, environment
3. **One issue per report** - Don't combine multiple bugs
4. **Search first** - Check if issue already exists

### Security Issues

1. **Responsible disclosure** - Report privately first if severe
2. **Provide details** - Impact, affected versions, mitigation
3. **Don't exploit** - Report, don't abuse

### Account Security

1. **Protect your secret key** - Never share or commit it
2. **Use environment variables** - Don't hardcode keys
3. **Verify signatures** - Ensure you're signing correct messages

### Maximizing Rewards

1. **Quality over quantity** - Valid issues count, not total
2. **Consistent participation** - 24h rolling window favors daily activity
3. **Target real issues** - Maintainers reject low-quality reports
4. **Read documentation** - Understand the project before reporting

---

## Troubleshooting

### "Invalid signature" Error

- Ensure you're using the correct secret key
- Check that timestamp is current (within 5 minutes)
- Verify the message format is correct

### "Username already registered" Error

- Each GitHub username can only link to one hotkey
- Contact support to change registration

### "Hotkey already registered" Error

- Each hotkey can only link to one GitHub username
- Use a different hotkey or contact support

### Issue Not Credited

- Verify issue has `valid` label
- Check issue was created by your registered GitHub account
- Ensure issue was resolved within last 24 hours
- Wait a few minutes for validators to scan

---

## Support

- **Documentation**: This file and `/docs/` directory
- **Issues**: [GitHub Issues](https://github.com/PlatformNetwork/bounty-challenge/issues)
- **Discord**: [Cortex Foundation Discord](https://discord.gg/cortexfoundation)
