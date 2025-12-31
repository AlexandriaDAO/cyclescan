# CycleScan Data Collection

**Last Updated:** 2025-12-30
**Status:** ✅ Production Ready - 3,138 canisters imported

---

## Directory Structure

```
data/
├── Core Data Files (Production)
│   ├── import_ready.json          # 2,638 blackhole canisters (with proxy_type)
│   ├── sns_canisters.json         # 503 SNS canisters
│   ├── project_mappings.json      # 433 identified projects (ODIN, etc.)
│   ├── canister_projects.csv      # Spreadsheet export of all research
│   ├── research_results.json      # Raw research output (744 tokens)
│   ├── token_names.txt            # Token reference list
│   └── import.candid              # Candid interface definition
│
├── Scripts (Data Collection & Processing)
│   ├── fetch_batch.sh             # Fetch raw canister data from IC Dashboard
│   ├── extract_trackable.sh       # Filter for blackhole canisters
│   ├── fetch_sns.py / .sh         # Fetch SNS canisters
│   ├── import_batch.sh            # Import canisters to backend
│   ├── research_canisters.py      # Research canister projects
│   ├── research_new_canisters.py  # Automated token research
│   ├── consolidate_results.py     # Consolidate research data
│   └── extract_tokens.py          # Extract ICRC-1 token info
│
├── Documentation
│   ├── CLAUDE.md                  # This file - data folder guide
│   ├── DATA_AUDIT_REPORT.md       # Cleanup audit results
│   ├── IMPORT_COMPLETE.md         # Backend import summary
│   ├── MASTER_CANISTER_RESEARCH.md # Research methodology & stats
│   ├── RESEARCH_COMPLETE.md       # Research completion notes
│   └── RESEARCH_PLAN.md           # Research methodology
│
└── archive/
    ├── FETCH_REMAINING_DATA.md    # Original fetch instructions
    └── raw_data/
        ├── canisters_*.json       # 40 raw API response files (0-985k)
        └── public_canisters_*.json # 40 filtered blackhole files
```

---

## Core Data Files

### 1. `import_ready.json` (351KB)
**Purpose:** Canonical source of all blackhole canisters
**Format:**
```json
[
  {
    "canister_id": "abc-cai",
    "proxy_id": "e3mmv-5qaaa-aaaah-aadma-cai",
    "proxy_type": "Blackhole"
  }
]
```
**Count:** 2,638 canisters
**Controllers:** ninegua blackhole, NNS Root

### 2. `sns_canisters.json` (66KB)
**Purpose:** All canisters from deployed SNSes
**Format:**
```json
[
  {
    "canister_id": "abc-cai",
    "proxy_id": "zxeu2-7aaaa-aaaaq-aaafa-cai",
    "proxy_type": "SnsRoot"
  }
]
```
**Count:** 503 canisters
**Source:** SNS-W registry (qaa6y-5yaaa-aaaaa-aaafa-cai)

### 3. `project_mappings.json` (198KB)
**Purpose:** Manual research - maps canister IDs to projects
**Format:**
```json
{
  "icrc1_tokens": {
    "canister-id": {
      "name": "Token Name",
      "symbol": "SYMBOL",
      "project": "ODIN.fun",
      "is_token": true
    }
  },
  "summary": {
    "total_canisters": 2638,
    "icrc1_tokens": 744,
    "identified_projects": 433
  }
}
```
**Projects Identified:**
- ODIN.fun: 391 tokens (Bioniq's Bitcoin Runes launchpad)
- Ordi Trade: 23 tokens
- FomoWell: 19 tokens
- Unknown tokens: 311

### 4. `canister_projects.csv` (280KB)
**Purpose:** Spreadsheet-friendly export
**Columns:** canister_id, name, symbol, project, is_token, proxy_id, proxy_type

### 5. `research_results.json` (571KB)
**Purpose:** Raw output from automated research
**Contains:** 2,638 entries with ICRC-1 token data, project mappings

---

## Data Sources & Collection

### Blackhole Canisters (2,638)
Canisters with ninegua or NNS Root as controller.

**Fetch Process:**
```bash
# Fetch raw data from IC Dashboard API
./fetch_batch.sh 0 25000

# Extract blackhole canisters (ninegua + NNS Root controllers)
./extract_trackable.sh canisters_0_25000.json

# Combine all ranges
jq -s 'add | unique_by(.canister_id)' public_canisters_*.json > trackable_canisters.json

# Add proxy_type field for import
jq '[.[] | . + {proxy_type: "Blackhole"}]' trackable_canisters.json > import_ready.json
```

**Coverage:** Offsets 0-985,000 (complete IC Dashboard data)

### SNS Canisters (503)
All canisters from deployed SNS DAOs.

**Fetch Process:**
```bash
# Fetch all SNS canisters
python3 fetch_sns.py
```

**Internal Commands:**
```bash
# List all SNSes
dfx canister --network ic call qaa6y-5yaaa-aaaaa-aaafa-cai list_deployed_snses '(record {})'

# Get canisters for specific SNS
dfx canister --network ic call <sns_root_id> get_sns_canisters_summary '(record {})'
```

### Project Research (433 identified)
Automated + manual research to identify projects.

**Research Process:**
```bash
# Automated token research
python3 research_new_canisters.py

# Consolidate results
python3 consolidate_results.py
```

**Methods:**
1. **ICRC-1 Query:** Call `icrc1_name()` and `icrc1_symbol()`
2. **Pattern Matching:** ODIN tokens end with "•ID•XXXX•ODIN"
3. **Web Search:** GitHub, forums, documentation
4. **Dashboard Lookup:** Check ICP Dashboard metadata

---

## Backend Import

**Backend Canister:** `vohji-riaaa-aaaac-babxq-cai`

### Import Process
```bash
# Combines blackhole + SNS canisters and imports in batches
./import_batch.sh
```

**What it does:**
1. Splits 3,138 canisters into batches of 200
2. Converts to Candid format
3. Calls `import_canisters()` on backend
4. Sets project names for identified canisters

**Current State:**
- ✅ 3,138 canisters imported
- ✅ 433 projects assigned
- ✅ Ready for snapshot collection

---

## Proxy Types

| Type | Query Method | Data Source | Count |
|------|--------------|-------------|-------|
| `Blackhole` | `canister_status(canister_id)` | IC Dashboard API + filter | 2,638 |
| `SnsRoot` | `get_sns_canisters_summary()` | SNS-W registry | 503 |

### Public Blackhole Controllers

| Name | ID | Notes |
|------|-----|-------|
| ninegua | `e3mmv-5qaaa-aaaah-aadma-cai` | Original blackhole canister |
| NNS Root | `r7inp-6aaaa-aaaaa-aaabq-cai` | NNS system canister |

**Note:** CycleOps V1/V2/V3 are private (reject non-CycleOps callers)

---

## Key Canister IDs

| Purpose | ID |
|---------|-----|
| CycleScan Backend | `vohji-riaaa-aaaac-babxq-cai` |
| SNS-W (SNS registry) | `qaa6y-5yaaa-aaaaa-aaafa-cai` |
| ninegua blackhole | `e3mmv-5qaaa-aaaah-aadma-cai` |
| NNS Root | `r7inp-6aaaa-aaaaa-aaabq-cai` |

---

## Statistics

### Current Dataset
| Metric | Count |
|--------|-------|
| **Total Canisters** | **3,138** |
| Blackhole | 2,638 |
| SNS | 503 |
| Duplicates filtered | 3 |

### Project Research
| Category | Count |
|----------|-------|
| **ICRC-1 Tokens** | **744** (28.2%) |
| Identified Projects | 433 (16.4%) |
| Unknown Tokens | 311 |
| Unknown Non-tokens | 1,894 (71.8%) |

### Identified Platforms
| Platform | Count | Description |
|----------|-------|-------------|
| ODIN.fun | 391 | Bioniq's Bitcoin Runes launchpad |
| Ordi Trade | 23 | Ordinals trading platform |
| FomoWell | 19 | Token launchpad |

---

## Common Tasks

### Refresh SNS Data
```bash
python3 fetch_sns.py
# Updates sns_canisters.json with latest SNS canisters
```

### Add More Blackhole Canisters
```bash
# If more canisters are created, fetch new range
./fetch_batch.sh 1000000 1025000
./extract_trackable.sh canisters_1000000_1025000.json

# Merge with existing data
jq -s 'add | unique_by(.canister_id)' import_ready.json public_canisters_1000000_1025000.json > updated_import_ready.json
```

### Research New Canisters
```bash
# Run automated token research
python3 research_new_canisters.py

# Consolidate results
python3 consolidate_results.py
```

### Import to Backend
```bash
# Import new canisters
./import_batch.sh

# Set project names
# (Script auto-generated during consolidation)
./update_projects.sh
```

---

## Adding New Data Sources

To add a new proxy type (e.g., CycleOps, custom blackhole):

1. **Create fetch script:** `fetch_<source>.py`
2. **Output format:**
   ```json
   [
     {
       "canister_id": "abc-cai",
       "proxy_id": "proxy-canister-id",
       "proxy_type": "NewType"
     }
   ]
   ```
3. **Update backend:** Add variant to `ProxyType` enum in `lib.rs`
4. **Add query logic:** Implement query method in backend
5. **Document:** Update this file with new type

---

## Archive

Raw data files (440MB) are archived in `archive/raw_data/`:
- `canisters_*.json` - 40 files with raw IC Dashboard responses
- `public_canisters_*.json` - 40 files with filtered blackhole canisters

**Can regenerate if needed** using archived raw data files.

---

## Maintenance

### File Sizes (Approximate)
- Core data: ~2MB (essential files)
- Archive: ~440MB (raw data, can be backed up remotely)
- Scripts: ~100KB
- Documentation: ~500KB

### Backup Strategy
1. **Critical (backup daily):**
   - `import_ready.json`
   - `sns_canisters.json`
   - `project_mappings.json`
   - `research_results.json`

2. **Important (backup weekly):**
   - All scripts
   - Documentation

3. **Archive (backup once, store remotely):**
   - `archive/raw_data/` (can regenerate but takes hours)
