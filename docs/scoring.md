# Scoring & Mathematics

Complete specification of the scoring system for Bounty Challenge.

## Overview

The scoring system is designed to:
1. Reward legitimate bug hunters fairly
2. Prevent gaming through mass low-quality submissions
3. Maintain incentive for continued participation

## Bounty Score Formula

### Individual Score

Each miner's score is calculated using logarithmic scaling:

$$S = \frac{\ln(1 + n)}{\ln(2) \times 10}$$

Where:
- $S$ = Miner's bounty score
- $n$ = Total number of valid issues claimed
- $\ln$ = Natural logarithm

### Why Logarithmic?

The logarithmic function provides **diminishing returns**:

| Valid Issues | Score | Marginal Gain |
|--------------|-------|---------------|
| 1 | 0.100 | +0.100 |
| 2 | 0.158 | +0.058 |
| 3 | 0.200 | +0.042 |
| 4 | 0.232 | +0.032 |
| 5 | 0.258 | +0.026 |
| 10 | 0.346 | +0.018/issue |
| 20 | 0.433 | +0.009/issue |
| 50 | 0.565 | +0.004/issue |
| 100 | 0.666 | +0.002/issue |

This prevents:
- **Spam attacks**: Filing 100 issues gives only ~6.6x the reward of 1 issue
- **Sybil attacks**: Splitting across accounts doesn't increase total reward
- **Quality dilution**: High effort for marginal gains after ~10 issues

## Weight Calculation

### Normalized Weights

Weights are calculated proportionally to scores:

$$w_i = \frac{S_i}{\sum_{j=1}^{N} S_j}$$

Where:
- $w_i$ = Weight for miner $i$
- $S_i$ = Score for miner $i$
- $N$ = Total number of miners with valid bounties

### Example

Given three miners:

| Miner | Valid Issues | Score |
|-------|--------------|-------|
| A | 10 | 0.346 |
| B | 5 | 0.258 |
| C | 2 | 0.158 |

Total score: $0.346 + 0.258 + 0.158 = 0.762$

Weights:
- $w_A = 0.346 / 0.762 = 0.454$ (45.4%)
- $w_B = 0.258 / 0.762 = 0.339$ (33.9%)
- $w_C = 0.158 / 0.762 = 0.207$ (20.7%)

## Bittensor Weight Conversion

For submission to Bittensor, weights are converted to u16:

$$W_i = \lfloor w_i \times 65535 \rfloor$$

Using the example above:
- $W_A = 29,751$
- $W_B = 22,211$
- $W_C = 13,573$

## Validation Requirements

An issue must meet ALL criteria to count:

1. **Closed**: Issue state is `closed`
2. **Valid Label**: Has label named `valid` (case-insensitive)
3. **Author Match**: Issue author matches registered GitHub username
4. **Unclaimed**: Issue hasn't been claimed by another miner

## Edge Cases

### No Valid Issues

If a miner has no valid issues:
- Score = 0
- Weight = 0
- Not included in weight calculation

### Single Miner

If only one miner has valid issues:
- Weight = 1.0 (100% of emissions)

### Tie Breaker

Miners with identical scores receive identical weights (no tie-breaking needed since weights are proportional).

## Code Reference

The scoring implementation in `src/challenge.rs`:

```rust
fn calculate_score(&self, valid_issues: u32) -> f64 {
    // score = log2(1 + valid_issues) / 10
    ((1.0 + valid_issues as f64).ln() / std::f64::consts::LN_2) / 10.0
}
```

## Simulation

Score curve visualization:

```
Score
  │
0.7├────────────────────────────────────────────────●
   │                                          ●
0.6├                                    ●
   │                              ●
0.5├                        ●
   │                  ●
0.4├            ●
   │       ●
0.3├    ●
   │  ●
0.2├●
   │
0.1├
   │
  0└──────────────────────────────────────────────────
    0    10   20   30   40   50   60   70   80   90  100
                      Valid Issues
```
