# Master Canister Research Documentation

**Date:** December 30, 2025
**Status:** IN PROGRESS
**Total Canisters:** 2,638 (206 original + 2,433 new)

---

## Overview

This document contains comprehensive research on all trackable canisters controlled by ninegua blackhole and NNS Root.

### Breakdown by Controller

| Controller | Canister ID | Count |
|------------|-------------|-------|
| ninegua blackhole | e3mmv-5qaaa-aaaah-aadma-cai | 1,092 |
| NNS Root | r7inp-6aaaa-aaaaa-aaabq-cai | 1,546 |
| **Total** | | **2,638** |

### Research Progress

| Batch | Count | Status | Tokens Found | Unknown |
|-------|-------|--------|--------------|---------|
| Original (0-200k) | 206 | ✅ Complete | 44 | 162 |
| New (200k-1M) | 2,433 | 🔄 In Progress | TBD | TBD |
| **Total** | **2,638** | | | |

---

## Research Methodology

For each canister:
1. ✅ **ICRC-1 Token Query** - Check `icrc1_name()` and `icrc1_symbol()`
2. ✅ **Frontend Check** - Test `.icp0.io`, `.ic0.app`, `.raw.icp0.io`
3. ⚠️ **Candid Interface** - Attempt `__get_candid_interface_tmp_hack()`
4. ✅ **ICP Dashboard** - Review controller and metadata
5. ❌ **Web Search** - Search GitHub, forums, general web

---

## Consolidated Results

*This section will be updated when research completes*

### Summary Statistics

- **Total Canisters:** 2,638
- **Identified Projects:** 433 (16.4%)
- **ICRC-1 Tokens:** 744 (28.2%)
  - ODIN.fun: 391
  - Unknown Tokens: 311
- **Unknown (Non-token):** 1,894 (71.8%)

**Last Updated:** 2025-12-30 10:11:54

---

## Detailed Canister Directory

*Format: `canister_id | project | type | notes`*

### Original Batch (206 canisters)

See `RESEARCH_PLAN.md` for detailed original batch results.

### New Batch (2,433 canisters - In Progress)

Research in progress. Results will be consolidated from `research_results.json`.

---

## Export Formats

### JSON Export (project_mappings.json)

```json
{
  "by_canister": {
    "canister-id": "Project Name"
  },
  "icrc1_tokens": {
    "canister-id": {
      "name": "Token Name",
      "symbol": "SYMBOL",
      "project": "Platform Name"
    }
  }
}
```

### CSV Export (canister_projects.csv)

```csv
canister_id,project,type,token_name,token_symbol,notes
abc-cai,ODIN.fun,token,EXAMPLE,EX,Token: EXAMPLE (EX)
def-cai,Unknown,backend,,, No token interface
```

---

## Files

- `MASTER_CANISTER_RESEARCH.md` - This file (master documentation)
- `project_mappings.json` - JSON database of all mappings
- `canister_projects.csv` - CSV export for easy import
- `RESEARCH_PLAN.md` - Original batch detailed research
- `research_results.json` - New batch raw results
- `RESEARCH_COMPLETE.md` - Original batch summary

---

*Last Updated: Research in progress - updates every 10 minutes*
