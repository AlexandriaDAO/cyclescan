# Data Folder Audit Report

## Summary

**Status:** ⚠️ **Needs Cleanup** - Multiple redundant files, format inconsistencies, and temporary files present

**Total Size:** 440M
**Key Issue:** Format inconsistency between `trackable_canisters.json` (missing proxy_type) and `import_ready.json` (has proxy_type)

---

## Core Data Files (KEEP)

These are the permanent foundation files:

| File | Size | Count | Status | Notes |
|------|------|-------|--------|-------|
| `trackable_canisters.json` | 271K | 2,638 | ❌ **Needs Fix** | Missing `proxy_type` field |
| `import_ready.json` | 351K | 2,638 | ✅ **Correct** | Has proper format with `proxy_type` |
| `sns_canisters.json` | 66K | 503 | ✅ **Good** | SNS data source |
| `project_mappings.json` | 15K | - | ✅ **Good** | Manual research (curated) |
| `canisters_*_*.json` (40 files) | ~440M | 935,526 | ✅ **Good** | Raw API data (archival) |
| `public_canisters_*_*.json` (40 files) | ~300K | 2,638 | ⚠️ **Consider** | Intermediate - can regenerate |

**Recommendation:** 
- Fix `trackable_canisters.json` to include `proxy_type` field OR delete it and use `import_ready.json` as the canonical source
- Consider archiving raw `canisters_*.json` files after import is complete

---

## Redundant Files (DELETE)

| File | Size | Count | Issue |
|------|------|-------|-------|
| `blackhole_import.json` | 35K | 262 | Old format - superseded by `import_ready.json` |
| `all_canisters.json` | 417K | 3,141 | Unclear purpose - overlaps with other files |
| `batch_*.json` (16 files) | ~350K | - | Import staging files - likely temporary |
| `import_candid.txt` | 472K | - | Unclear purpose |

---

## Scripts (KEEP)

| File | Purpose | Status |
|------|---------|--------|
| `fetch_batch.sh` | Fetch raw canister data | ✅ Essential |
| `extract_trackable.sh` | Filter trackable canisters | ✅ Essential |
| `fetch_sns.py` / `.sh` | Fetch SNS data | ✅ Essential |
| `import_batch.sh` | Import to canister | ✅ Essential |
| `research_canisters.py` | Project research | ✅ Useful |
| `research_new_canisters.py` | New project research | ✅ Useful |
| `consolidate_results.py` | Consolidate research | ✅ Useful |
| `monitor_and_report.sh` | Monitor progress | ⚠️ Consider keeping |
| `extract_tokens.py` | Extract token info | ✅ Useful |
| `deep_research.py` | Deep research | ✅ Useful |

---

## Documentation (KEEP/ORGANIZE)

| File | Status | Notes |
|------|--------|-------|
| `CLAUDE.md` | ✅ Keep | Main docs |
| `FETCH_REMAINING_DATA.md` | ⚠️ Archive | Task now complete - obsolete |
| `MASTER_CANISTER_RESEARCH.md` | ✅ Keep | Research notes |
| `RESEARCH_COMPLETE.md` | ✅ Keep | Research summary |
| `RESEARCH_PLAN.md` | ✅ Keep | Research methodology |
| `import.candid` | ✅ Keep | Candid interface |
| `token_names.txt` | ✅ Keep | Token reference |

---

## Logs & Temp Files (DELETE)

| File | Size | Issue |
|------|------|-------|
| `completion_monitor.log` | 0 | Empty log file |
| `research_output.log` | 50K | Old log file |
| `research_new_output.log` | 0 | Empty log file |
| `research_results.json` | 222K | Intermediate - likely consolidated elsewhere |
| `__pycache__/` | - | Python cache directory |

---

## Critical Issues

### 1. Format Inconsistency ⚠️
**Problem:** `trackable_canisters.json` lacks `proxy_type` field

```json
// trackable_canisters.json (WRONG)
{
  "canister_id": "abc-cai",
  "proxy_id": "e3mmv-5qaaa-aaaah-aadma-cai"
}

// import_ready.json (CORRECT)
{
  "canister_id": "abc-cai",
  "proxy_id": "e3mmv-5qaaa-aaaah-aadma-cai",
  "proxy_type": "Blackhole"
}
```

**Solution:** Regenerate `trackable_canisters.json` with proper format OR use `import_ready.json` as canonical

### 2. Unclear Purpose Files
- `all_canisters.json` (3,141 canisters) - What is this? SNS + Blackhole combined?
- `batch_*.json` - Import staging? Can we delete after import?
- `import_candid.txt` - What is this?

### 3. Intermediate Files
- `public_canisters_*_*.json` (40 files) - Can be regenerated from raw data if needed

---

## Recommended Cleanup Actions

### Phase 1: Fix Critical Issues
```bash
# 1. Fix trackable_canisters.json format
# Option A: Regenerate from public_canisters_*.json
jq -s 'add | unique_by(.canister_id) | map(. + {proxy_type: "Blackhole"})' public_canisters_*.json > trackable_canisters.json

# Option B: Use import_ready.json as the canonical file
cp import_ready.json trackable_canisters.json
```

### Phase 2: Remove Redundant Files
```bash
# Remove old/redundant files
rm blackhole_import.json
rm all_canisters.json  # Verify purpose first
rm batch_*.json  # After confirming import complete
rm import_candid.txt  # Verify purpose first
rm research_results.json  # If consolidated elsewhere
```

### Phase 3: Clean Logs & Temp
```bash
rm *.log
rm -rf __pycache__/
```

### Phase 4: Archive Documentation
```bash
mkdir -p archive/
mv FETCH_REMAINING_DATA.md archive/  # Task complete
```

### Phase 5: Optional - Archive Raw Data
```bash
# After successful import, consider archiving raw files
mkdir -p archive/raw_data/
mv canisters_*_*.json archive/raw_data/
mv public_canisters_*_*.json archive/raw_data/
```

---

## Final Clean Structure

```
data/
├── trackable_canisters.json    # Canonical blackhole canisters (with proxy_type)
├── sns_canisters.json           # SNS canisters
├── project_mappings.json        # Manual project research
├── import.candid                # Candid interface
├── token_names.txt              # Token reference
├── CLAUDE.md                    # Main documentation
├── *.sh                         # Essential scripts
├── *.py                         # Essential scripts
└── archive/                     # Archived files
    ├── FETCH_REMAINING_DATA.md
    └── raw_data/
        ├── canisters_*_*.json
        └── public_canisters_*_*.json
```

---

## Data Quality: ✅ Good

- **Uniform format:** Raw data is consistent (canisters_*.json)
- **Complete coverage:** 0-985k offsets (all available data)
- **No duplicates:** `unique_by(.canister_id)` applied
- **Proper filtering:** Blackhole controllers verified
- **SNS data:** Separate, clean source

## Space Savings

- Remove redundant: ~1.5MB
- Remove logs/temp: ~300KB
- Archive raw data: ~440MB → frees up working dir

**Total Potential Cleanup:** ~442MB
