# CycleScan Deployment Plan

**Date:** 2025-12-30  
**Status:** Backend Ready - Needs Deployment & Snapshot Collection  
**Priority:** HIGH - Deploy and verify production readiness

---

## Current State Summary

### ✅ Completed
- **Data Collection:** 3,138 canisters tracked (2,638 blackhole + 503 SNS)
- **Project Research:** 433 projects identified (ODIN.fun, Ordi Trade, FomoWell)
- **Backend Import:** All canisters imported with project names
- **Documentation:** Complete data folder guide in `data/CLAUDE.md`

### 🎯 Your Mission
Deploy the updated backend with 3,138 canisters and take the first snapshot to start tracking cycles burned.

---

## Task Overview

1. **Deploy Backend** (~5 min)
2. **Verify Import** (~2 min)
3. **Take Initial Snapshot** (~10 min)
4. **Verify Leaderboard** (~2 min)
5. **Set Up Recurring Snapshots** (~10 min)

**Total Time:** ~30 minutes  
**Complexity:** Medium - Straightforward but requires monitoring

---

## Prerequisites

### Required Files (all present in project)
- ✅ `src/cyclescan_backend/src/lib.rs` - Backend code
- ✅ `src/cyclescan_backend/Cargo.toml` - Dependencies
- ✅ `src/cyclescan_backend/cyclescan_backend.did` - Candid interface
- ✅ `dfx.json` - Project configuration
- ✅ `data/import_ready.json` - 2,638 blackhole canisters
- ✅ `data/sns_canisters.json` - 503 SNS canisters
- ✅ `data/project_mappings.json` - Project research

### Current Backend State
```
Canister ID: vohji-riaaa-aaaac-babxq-cai
Current Stats:
  - Canisters: 3,138
  - Snapshots: 35,792 (historical)
  - Projects: 433 set
```

### Environment
- Working Directory: `/home/theseus/alexandria/cyclescan`
- Data Directory: `/home/theseus/alexandria/cyclescan/data`
- Identity: `daopad`
- Network: `ic` (mainnet)

---

## Task 1: Deploy Backend

**Objective:** Deploy latest backend code with all imports

### Steps

1. **Navigate to project root**
   ```bash
   cd /home/theseus/alexandria/cyclescan
   ```

2. **Verify code is ready**
   ```bash
   # Check if backend builds
   cargo check --manifest-path src/cyclescan_backend/Cargo.toml
   ```

3. **Deploy to mainnet**
   ```bash
   ./scripts/deploy.sh
   ```
   
   **Expected output:**
   ```
   Deploying canister cyclescan_backend...
   Installing code for canister cyclescan_backend...
   Module hash: [hash]
   ```

   **Time:** 2-3 minutes

4. **Verify deployment**
   ```bash
   dfx canister call cyclescan_backend get_stats --network ic
   ```
   
   **Expected output:**
   ```
   record {
     canister_count = 3_138 : nat64;
     snapshot_count = [number] : nat64;
   }
   ```

### ✅ Success Criteria
- Deployment completes without errors
- `get_stats` shows 3,138 canisters
- All imports preserved after deployment

### ⚠️ Troubleshooting
- If deployment fails: Check Cargo.toml dependencies
- If canister_count is 0: Data was cleared, re-run import (see Recovery section)

---

## Task 2: Verify Import

**Objective:** Confirm all 3,138 canisters and 433 projects are present

### Steps

1. **Check canister count**
   ```bash
   dfx canister call cyclescan_backend get_stats --network ic
   ```
   
   **Expected:** `canister_count = 3_138`

2. **Verify project names (sample check)**
   ```bash
   # Check a known ODIN.fun canister
   dfx canister call cyclescan_backend get_leaderboard --network ic | head -20
   ```
   
   **Expected:** Should see canisters in the response (even if cycles are 0 before snapshot)

### ✅ Success Criteria
- 3,138 canisters present
- Leaderboard call succeeds (may be empty before snapshot)

### ⚠️ If Data Missing
See "Recovery: Re-import Data" section below

---

## Task 3: Take Initial Snapshot

**Objective:** Query all 3,138 canisters to get current cycle counts

### Steps

1. **Initiate snapshot**
   ```bash
   echo "Starting snapshot at $(date)"
   dfx canister call cyclescan_backend take_snapshot --network ic
   ```
   
   **This will:**
   - Query 2,638 blackhole canisters via `canister_status`
   - Query 503 SNS canisters via `get_sns_canisters_summary`
   - Store cycles data in stable memory
   - Prune snapshots older than 7 days

2. **Monitor progress**
   The call may take 5-10 minutes. Watch for:
   ```
   (
     variant {
       Ok = record {
         blackhole_success = [number] : nat64;
         blackhole_failed = [number] : nat64;
         sns_success = [number] : nat64;
         sns_failed = [number] : nat64;
       }
     }
   )
   ```

3. **Expected Results**
   - `blackhole_success`: ~2,630+ (most should succeed)
   - `blackhole_failed`: <10 (some canisters may be unreachable)
   - `sns_success`: ~500+ (most SNS queries succeed)
   - `sns_failed`: <5

### ✅ Success Criteria
- Snapshot completes successfully
- >90% of canisters return data
- No timeout errors

### ⚠️ Troubleshooting
- **Timeout:** Snapshot call timed out - This is NORMAL for first run
  - The snapshot IS still running in the background
  - Wait 10 minutes, then check `get_stats` again
  - `snapshot_count` should increase
  
- **High failure rate (>20%):** Possible canister issues or network problems
  - Check a few failed canisters manually
  - Re-run snapshot after 5 minutes

---

## Task 4: Verify Leaderboard

**Objective:** Confirm cycles data is being tracked and leaderboard works

### Steps

1. **Get leaderboard**
   ```bash
   dfx canister call cyclescan_backend get_leaderboard --network ic
   ```
   
   **Expected output:**
   ```
   (
     vec {
       record {
         canister_id = principal "abc-cai";
         project_name = opt "ODIN.fun";
         cycles_burned_1h = [number] : nat;
         cycles_burned_24h = [number] : nat;
         cycles_burned_7d = [number] : nat;
       };
       ...
     }
   )
   ```

2. **Check stats again**
   ```bash
   dfx canister call cyclescan_backend get_stats --network ic
   ```
   
   **Expected:**
   - `snapshot_count` should be higher than before
   - `newest_snapshot` should be recent timestamp

### ✅ Success Criteria
- Leaderboard returns data
- Top canisters show cycle burn rates
- Project names appear correctly

---

## Task 5: Set Up Recurring Snapshots

**Objective:** Automate hourly snapshot collection

### Option A: GitHub Actions (Recommended)

1. **Create workflow file**
   ```bash
   mkdir -p .github/workflows
   cat > .github/workflows/snapshot.yml << 'YAML'
   name: Take Hourly Snapshot
   
   on:
     schedule:
       - cron: '0 * * * *'  # Every hour
     workflow_dispatch:  # Allow manual trigger
   
   jobs:
     snapshot:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v3
         
         - name: Install dfx
           run: |
             sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"
             echo "$HOME/.local/share/dfx/bin" >> $GITHUB_PATH
         
         - name: Setup identity
           run: |
             echo "${{ secrets.DFX_IDENTITY }}" > identity.pem
             dfx identity import github identity.pem
             dfx identity use github
         
         - name: Take snapshot
           run: |
             dfx canister call cyclescan_backend take_snapshot --network ic
   YAML
   ```

2. **Add identity secret**
   - Go to GitHub repo → Settings → Secrets
   - Add `DFX_IDENTITY` with content of `~/.config/dfx/identity/daopad/identity.pem`

3. **Test manual trigger**
   - Go to Actions tab
   - Select "Take Hourly Snapshot"
   - Click "Run workflow"

### Option B: Cron Job (Local/VPS)

1. **Create snapshot script**
   ```bash
   cat > scripts/take_snapshot.sh << 'BASH'
   #!/bin/bash
   set -e
   
   cd /home/theseus/alexandria/cyclescan
   export DFX_WARNING=-mainnet_plaintext_identity
   
   echo "$(date): Taking snapshot..."
   dfx canister call cyclescan_backend take_snapshot --network ic
   echo "$(date): Snapshot complete"
   BASH
   
   chmod +x scripts/take_snapshot.sh
   ```

2. **Add to crontab**
   ```bash
   crontab -e
   # Add this line:
   0 * * * * /home/theseus/alexandria/cyclescan/scripts/take_snapshot.sh >> /tmp/cyclescan-snapshots.log 2>&1
   ```

3. **Test manually**
   ```bash
   ./scripts/take_snapshot.sh
   ```

### ✅ Success Criteria
- First automated snapshot runs successfully
- Can verify in logs/GitHub Actions
- Stats show increasing snapshot_count

---

## Recovery: Re-import Data

**If deployment cleared the data (canister_count = 0):**

```bash
cd /home/theseus/alexandria/cyclescan/data

# Re-import all canisters
./import_batch.sh

# Verify count
dfx canister call cyclescan_backend get_stats --network ic
# Should show: canister_count = 3_138

# Re-set project names
./update_projects.sh  # If this file exists
# OR regenerate it:
python3 << 'EOF'
import json

with open('project_mappings.json', 'r') as f:
    mappings = json.load(f)

tokens = mappings.get('icrc1_tokens', {})
for canister_id, data in tokens.items():
    project = data.get('project')
    if project:
        print(f'dfx canister call cyclescan_backend set_project \'(principal "{canister_id}", opt "{project}")\' --network ic')
