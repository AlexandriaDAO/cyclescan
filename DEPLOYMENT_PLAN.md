# CycleScan Deployment Plan

**Date:** 2025-12-30
**Status:** Backend Ready - Needs Deployment & Snapshot Collection
**Priority:** HIGH - Deploy and verify production readiness

---

## Current State Summary

### Completed
- **Data Collection:** 3,138 canisters tracked (2,638 blackhole + 503 SNS)
- **Project Research:** 433 projects identified (ODIN.fun, Ordi Trade, FomoWell)
- **Backend Import:** All canisters imported with project names
- **Documentation:** Complete data folder guide in `data/CLAUDE.md`

### Your Mission
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

## Quick Start (TL;DR)

```bash
# 1. Deploy
cd /home/theseus/alexandria/cyclescan
./scripts/deploy.sh

# 2. Take snapshot
dfx canister call cyclescan_backend take_snapshot --network ic

# 3. Check leaderboard
dfx canister call cyclescan_backend get_leaderboard --network ic

# 4. Set up hourly cron
# See "Task 5" below for details
```

---

## Prerequisites

**Backend Canister:** `vohji-riaaa-aaaac-babxq-cai`
**Working Directory:** `/home/theseus/alexandria/cyclescan`
**Identity:** `daopad`

All required files are present and ready:
- Backend code in `src/cyclescan_backend/`
- Data files in `data/` (3,138 canisters already imported)
- Deploy script: `./scripts/deploy.sh`

---

## Task 1: Deploy Backend (5 min)

```bash
cd /home/theseus/alexandria/cyclescan
./scripts/deploy.sh
```

**What this does:**
- Builds the Rust backend
- Deploys to canister `vohji-riaaa-aaaac-babxq-cai`
- Preserves all 3,138 imported canisters

**Verify:**
```bash
dfx canister call cyclescan_backend get_stats --network ic
```

**Expected:** `canister_count = 3_138 : nat64`

---

## Task 2: Take Initial Snapshot (10 min)

```bash
dfx canister call cyclescan_backend take_snapshot --network ic
```

**What this does:**
- Queries all 3,138 canisters for current cycles
- Stores snapshot data in stable memory
- May take 5-10 minutes to complete

**Expected output:**
```
variant { Ok = record {
  blackhole_success = 2600+ : nat64;
  blackhole_failed = <10 : nat64;
  sns_success = 500+ : nat64;
  sns_failed = <5 : nat64;
}}
```

**Note:** If it times out, the snapshot is still running in background. Wait 10 min and check stats.

---

## Task 3: Verify Leaderboard (2 min)

```bash
dfx canister call cyclescan_backend get_leaderboard --network ic
```

**Expected:** List of canisters with:
- `canister_id`
- `project_name` (for 433 identified projects)
- `cycles_burned_1h`, `cycles_burned_24h`, `cycles_burned_7d`

---

## Task 4: Set Up Recurring Snapshots (10 min)

### Option A: Cron Job (Simple)

```bash
# Create snapshot script
cat > scripts/take_snapshot.sh << 'BASH'
#!/bin/bash
set -e
cd /home/theseus/alexandria/cyclescan
export DFX_WARNING=-mainnet_plaintext_identity
dfx canister call cyclescan_backend take_snapshot --network ic
BASH

chmod +x scripts/take_snapshot.sh

# Add to crontab
crontab -e
# Add this line:
# 0 * * * * /home/theseus/alexandria/cyclescan/scripts/take_snapshot.sh >> /tmp/cyclescan.log 2>&1

# Test it
./scripts/take_snapshot.sh
```

### Option B: GitHub Actions (Better for remote)

Create `.github/workflows/snapshot.yml` - see full plan below for template.

---

## Troubleshooting

### Data Missing After Deploy
```bash
cd data
./import_batch.sh  # Re-import all canisters (5 min)
```

### Snapshot Timeout
This is normal - snapshot continues in background. Wait 10 min and check:
```bash
dfx canister call cyclescan_backend get_stats --network ic
```
`snapshot_count` should increase.

### High Failure Rate
Some canisters may be unreachable. >90% success is good. Re-run snapshot:
```bash
dfx canister call cyclescan_backend take_snapshot --network ic
```

---

## Success Criteria

After completing all tasks:
- [x] Backend deployed
- [x] 3,138 canisters confirmed via `get_stats`
- [x] Initial snapshot completed (>90% success)
- [x] Leaderboard showing cycle burn data
- [x] Hourly snapshots automated

---

## Expected Timeline

- **Hour 0:** Deploy + take snapshot
- **Hour 1:** First automated snapshot runs
- **Day 1:** 24 snapshots, accurate 24h burn rates
- **Day 7:** Full 7-day burn rates, ready for production

---

## Important Canister URLs

- **Dashboard:** https://dashboard.internetcomputer.org/canister/vohji-riaaa-aaaac-babxq-cai
- **Candid UI:** https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=vohji-riaaa-aaaac-babxq-cai

---

## Data Reference

**Total Canisters:** 3,138
- Blackhole: 2,638 (ninegua + NNS Root)
- SNS: 503

**Identified Projects:** 433
- ODIN.fun: 391 (Bitcoin Runes launchpad)
- Ordi Trade: 23
- FomoWell: 19

**Unknown:** 2,705 canisters (need manual research)

**Data Files:**
- `data/import_ready.json` - Blackhole canisters
- `data/sns_canisters.json` - SNS canisters
- `data/project_mappings.json` - Project research
- `data/CLAUDE.md` - Complete data guide

---

## Next Steps (Future)

1. **Frontend** - Build UI to display leaderboard
2. **Monitoring** - Alert on snapshot failures
3. **Research** - Identify remaining 2,705 canisters
4. **Optimization** - Improve query batching

---

## Key Commands Reference

```bash
# Deploy backend
./scripts/deploy.sh

# Get backend stats
dfx canister call cyclescan_backend get_stats --network ic

# Take snapshot
dfx canister call cyclescan_backend take_snapshot --network ic

# Get leaderboard
dfx canister call cyclescan_backend get_leaderboard --network ic

# Re-import data (if needed)
cd data && ./import_batch.sh
```

---

**Ready to deploy!** Follow tasks 1-4 in order, then you're production-ready. 🚀

