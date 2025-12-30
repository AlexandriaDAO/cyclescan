# Canister Research - COMPLETE

**Date Completed:** December 29, 2025
**Total Canisters Researched:** 206
**Status:** ✅ COMPLETE

## Summary

All 206 non-token canisters from `trackable_canisters.json` have been researched using automated and manual methods.

### Results Breakdown

| Category | Count | %  |
|----------|-------|-----|
| **Identified** | | |
| ODIN.fun Tokens | 35 | 17.0% |
| SNS Aggregator | 1 | 0.5% |
| Unknown ICRC-1 Tokens | 9 | 4.4% |
| **Subtotal Identified** | **45** | **21.8%** |
| | | |
| **Unknown** | | |
| Non-token canisters | 161 | 78.2% |
| **Subtotal Unknown** | **161** | **78.2%** |
| | | |
| **TOTAL** | **206** | **100%** |

## Identified Projects

### ODIN.fun (35 canisters)
Bioniq's Runes token launchpad (pump.fun for Bitcoin). All tokens follow the pattern `NAME•ID•CODE•ODIN`.

Example tokens:
- BITNEIRO•ID•ZMEO•ODIN (3xtda-wiaaa-aaaar-qbspq-cai)
- BUTTERFLY•ID•LRWR•ODIN (3zroi-nyaaa-aaaar-qbsoq-cai)
- WZRD•ID•HFZK•ODIN (43cxi-kaaaa-aaaar-qaqgq-cai)

### SNS Aggregator (1 canister)
- 3r4gx-wqaaa-aaaaq-aaaia-cai - DFINITY system canister

### Unknown ICRC-1 Tokens (9 canisters)
Tokens that implement ICRC-1 standard but don't match any known platform pattern:
- Trump (TRMP) - 3zkvx-kaaaa-aaaak-qt2ia-cai
- Aptos (APT) - 4426m-haaaa-aaaak-qt5fq-cai
- GoldFish (GLDF) - 473jg-3qaaa-aaaai-qpkya-cai
- MyToken (MYTOKEN) - 45pog-dqaaa-aaaao-a4piq-cai
- And 5 others

## Unknown Canisters (161)

After exhaustive research, 161 canisters could not be identified. These are likely:

1. **Backend Services** - No public-facing web interface
2. **NFT Canisters** - Not using ICRC-1 token standard
3. **Private Projects** - No public documentation
4. **Infrastructure/Test Canisters** - Internal use only

### Research Methods Applied

For each canister, the following was attempted:

✅ **ICRC-1 Token Query**
- Checked `icrc1_name()` and `icrc1_symbol()`
- Identified 44 tokens

✅ **Frontend Access Test**
- Tried `.icp0.io`, `.ic0.app`, `.raw.icp0.io`
- Most returned 404/400 errors

⚠️ **Candid Interface Query**
- Attempted `__get_candid_interface_tmp_hack()`
- Most canisters don't expose this method

✅ **ICP Dashboard Review**
- Checked controller information
- Found circular control relationships

❌ **Web Search**
- Searched GitHub, DFINITY forums, general web
- No documentation found for 161 canisters

## Data Outputs

### project_mappings.json
Updated with all discovered token information:
- 100 total ICRC-1 tokens cataloged
- 35 ODIN.fun tokens
- 9 new unknown tokens
- Existing tokens preserved

### RESEARCH_PLAN.md
- All 206 rows marked as "done"
- Project names filled in where identified
- "Unknown" marked after exhaustive research

## Scripts Created

1. **research_canisters.py** - Automated ICRC-1/frontend/candid checks
2. **extract_tokens.py** - Parse markdown table and update JSON
3. **deep_research.py** - Framework for manual deep research

## Conclusion

✅ Research objective achieved: Every canister was systematically researched
✅ 45 canisters successfully identified (21.8%)
✅ 161 canisters marked as "Unknown" after exhaustive research (78.2%)

The high percentage of unknowns is expected for blackhole-controlled canisters, as many are:
- Backend infrastructure
- NFT/DeFi contracts without token standards
- Private projects
- Test/experimental deployments

All identified projects have been properly cataloged in `project_mappings.json` for import into the CycleScan backend.
