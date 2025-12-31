# Plan: Import SNS Project Names into Research Results

## Overview

CycleScan tracks cycle consumption for ICP canisters. We have two data sources:
1. **Blackhole canisters** - Already researched with project names in `research_results.json`
2. **SNS canisters** - Collected in `sns_canisters.json` but **missing project names**

This plan describes how to extract project names from SNS data and add them to our finished research results.

---

## Current State

### research_results.json (2,433 entries)
Contains researched blackhole canisters with project identification:
```json
{
  "canister_id": "22ltm-7aaaa-aaaar-qbnnq-cai",
  "project": "ODIN.fun",
  "notes": "Token: BTHACD•ID•FQEE•ODIN",
  "is_token": true,
  "token_name": "BTHACD•ID•FQEE•ODIN",
  "token_symbol": "BTHACD•ID•FQEE•ODIN"
}
```

### sns_canisters.json (503 entries)
Contains SNS canisters but NO project names:
```json
{
  "canister_id": "zxeu2-7aaaa-aaaaq-aaafa-cai",
  "proxy_id": "zxeu2-7aaaa-aaaaq-aaafa-cai",
  "proxy_type": "SnsRoot"
}
```

---

## Goal

Add all 503 SNS canisters to `research_results.json` with their project names, so the backend can display meaningful names for SNS projects on the leaderboard.

---

## How to Get SNS Project Names

Each SNS has a **ledger canister** that exposes `icrc1_name()` which returns the project/token name.

### Step 1: Get SNS Registry
```bash
dfx canister --network ic call qaa6y-5yaaa-aaaaa-aaafa-cai list_deployed_snses '(record {})'
```

This returns all deployed SNSes with their canister IDs:
- `root_canister_id`
- `governance_canister_id`
- `index_canister_id`
- `swap_canister_id`
- `ledger_canister_id` <-- Use this to get the name

### Step 2: Get Project Name from Ledger
```bash
dfx canister --network ic call <ledger_canister_id> icrc1_name '()'
```

Example:
```bash
dfx canister --network ic call zfcdd-tqaaa-aaaaq-aaaga-cai icrc1_name '()'
# Returns: ("Draggin Karma Points")
```

### Step 3: Map Name to All Canisters in That SNS
Each SNS has multiple canisters (root, governance, ledger, index, swap, plus dapps and archives). All canisters belonging to the same SNS should share the same project name.

The `sns_canisters.json` file groups canisters by `proxy_id` (which is the SNS root canister ID).

---

## Implementation Approach

### Option A: Python Script (Recommended)
Create a script that:
1. Reads `sns_canisters.json` to get all SNS canisters grouped by proxy_id
2. Calls `list_deployed_snses` to get the ledger canister for each SNS root
3. Calls `icrc1_name()` on each ledger to get the project name
4. Creates entries for each canister with the project name
5. Appends to `research_results.json`

### Option B: Manual dfx Commands
For each of the 41 live SNSes:
1. Get ledger ID from registry
2. Call icrc1_name on ledger
3. Manually create entries

---

## Output Format

For each SNS canister, create an entry like:
```json
{
  "canister_id": "zxeu2-7aaaa-aaaaq-aaafa-cai",
  "project": "Draggin Karma Points",
  "notes": "SNS Root canister",
  "is_token": false
}
```

The `notes` field should indicate the canister role:
- "SNS Root canister"
- "SNS Governance canister"
- "SNS Ledger canister"
- "SNS Index canister"
- "SNS Swap canister"
- "SNS Dapp canister"
- "SNS Archive canister"

For ledger canisters specifically, set `is_token: true` since they are the token contract.

---

## Important Notes

### Dead SNSes
11 SNS projects are dead (out of cycles or no wasm). These are already excluded from `sns_canisters.json` because `get_sns_canisters_summary` failed on them during data collection. No action needed.

### Canister Roles
The `get_sns_canisters_summary` response includes the role of each canister:
- `root` - Root canister
- `governance` - Governance canister
- `ledger` - Ledger/token canister
- `index` - Index canister
- `swap` - Swap canister
- `dapps` - Array of dapp canisters
- `archives` - Array of archive canisters

Currently `sns_canisters.json` doesn't store the role, but it can be inferred by re-querying `get_sns_canisters_summary` or by matching against the registry.

### Avoiding Duplicates
Check that SNS canister IDs don't already exist in `research_results.json` before adding.

---

## Files to Modify

| File | Action |
|------|--------|
| `research_results.json` | Append SNS canister entries |
| `sns_canisters.json` | Read only (source data) |

---

## Verification

After completion:
1. `research_results.json` should have ~2,936 entries (2,433 + 503)
2. Each SNS project should have all its canisters with the same project name
3. Run: `jq '[.[] | select(.project | contains("SNS") or contains("Draggin") or contains("OpenChat"))] | length' research_results.json` to spot-check

---

## Reference: SNS Project Names (Sample)

| SNS Root | Ledger | Project Name |
|----------|--------|--------------|
| zxeu2-7aaaa-aaaaq-aaafa-cai | zfcdd-tqaaa-aaaaq-aaaga-cai | Draggin Karma Points |
| 3e3x2-xyaaa-aaaaq-aaalq-cai | 2ouva-viaaa-aaaaq-aaamq-cai | (query to get) |
| ... | ... | ... |

---

## Commands Reference

```bash
# List all SNSes
dfx canister --network ic call qaa6y-5yaaa-aaaaa-aaafa-cai list_deployed_snses '(record {})'

# Get project name from ledger
dfx canister --network ic call <ledger_id> icrc1_name '()'

# Get all canisters for an SNS
dfx canister --network ic call <root_id> get_sns_canisters_summary '(record {})'
```
