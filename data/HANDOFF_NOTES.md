# Website Research Handoff Notes

**Date:** 2025-12-31
**Status:** Incomplete - Needs Full Redo with Proper Verification

## What Happened

An initial attempt was made to research and upload websites for 79 projects (1,477 canisters). However, the research was rushed and **many websites are INCORRECT**:

### Confirmed Errors
1. **Orbit** - Listed as `https://orbithub.app/` (doesn't exist) → Correct: `https://orbit.global`
2. **ODIN.fun** - Listed as `https://internetcomputer.org` (wrong) → Correct: `https://odin.fun`
3. Many others likely wrong - not verified

### What Went Wrong
- ❌ Websites were not verified (didn't check if URLs actually load)
- ❌ Many were guessed or pattern-matched without confirmation
- ❌ Some defaulted to `internetcomputer.org` when they shouldn't have
- ❌ Single web search per project instead of thorough research
- ❌ No cross-referencing with official sources
- ❌ No source documentation for where websites were found

## Current State of Backend

**⚠️ WARNING:** The backend currently has ~1,477 canister website entries, but many are WRONG.

### What Needs to Happen
1. **Audit all 79 existing entries** - verify or correct each one
2. **Research remaining ~73 projects** properly
3. **Re-upload corrected data** to backend

## Files in This Directory

### Critical Files
- **`WEBSITE_RESEARCH_PLAN.md`** - COMPLETE execution plan for doing this right (323 lines)
  - Detailed methodology
  - Quality standards
  - Complete project list
  - Step-by-step process
  - Success criteria

- **`websites_for_import.json`** - ⚠️ DO NOT USE - Contains incorrect data
- **`batch_*_websites.sh`** - ⚠️ DO NOT RE-RUN - Already uploaded incorrect data

### Data Files
- **`extract_projects.py`** - Extracts unique projects from backend (GOOD - reuse this)
- **`current_canisters_export.txt`** - Backend export (may be outdated)

## Quick Start for Next Agent

1. **Read `WEBSITE_RESEARCH_PLAN.md`** - This is your complete guide
2. **Start with Phase 1: Audit and Fix**
   - Export current websites from backend
   - Verify/correct the 79 existing entries
   - Priority: Fix Orbit and ODIN.fun immediately

3. **Follow the methodology strictly:**
   - Multiple searches per project
   - WebFetch to verify URLs load
   - Document sources
   - Cross-reference with official channels
   - High confidence standards

4. **Batch upload corrections** as you verify them

## Backend API Reference

```bash
# Export all canisters with current websites
dfx canister --network ic call vohji-riaaa-aaaac-babxq-cai export_canisters

# Update websites in bulk (batches of 100 recommended)
dfx canister --network ic call vohji-riaaa-aaaac-babxq-cai set_websites \
  '(vec { record { principal "xxx-cai"; opt "https://example.com" } })'

# Get project-specific canisters
dfx canister --network ic call vohji-riaaa-aaaac-babxq-cai get_project_canisters \
  '("Project Name")'
```

## Projects Overview

- **Total:** 152 unique projects
- **Researched (but many wrong):** 79
- **Remaining:** 73
- **Likely no website:** ~20 (generic infrastructure, abandoned, unknown tokens)
- **Need thorough research:** ~53

## Key Projects to Prioritize

### Must Get Right (High Profile)
- All SNS DAOs (GOLDAO, ICVC, Sneed, ELNA, etc.)
- Major DEXs (Sonic, ICPSwap, Kong, ICLighthouse)
- Wallets (NFID, Oisy)
- Infrastructure (Juno, Orbit, ICExplorer)
- ODIN.fun (token launchpad with 391 tokens)

### Medium Priority
- Token projects with active communities
- DeFi platforms
- NFT marketplaces
- Social platforms

### Low Priority (may not have websites)
- Individual meme tokens
- Abandoned projects
- Generic "Unknown Token" entries
- Infrastructure canisters

## Success Metrics

When done correctly, you should achieve:
- ✅ 100% of projects researched with decision documented
- ✅ >95% confidence on all major projects
- ✅ All websites verified to load
- ✅ Sources documented for every entry
- ✅ No defaults to internetcomputer.org unless truly ICP infrastructure
- ✅ Comprehensive report with confidence levels

## Questions to Ask

If you encounter issues:
1. Is this a real project or infrastructure component?
2. Is the project active or abandoned?
3. Does the website actually load and match the project?
4. Have I checked at least 3 different sources?
5. Am I confident enough to mark this "high confidence"?

## Final Notes

This is a thorough, detail-oriented task that requires:
- **Patience** - Don't rush like the first attempt
- **Verification** - Check everything
- **Documentation** - Track your sources
- **Quality over speed** - Get it right

The plan document (`WEBSITE_RESEARCH_PLAN.md`) has everything you need. Follow it carefully and you'll deliver high-quality results.

Good luck! 🚀
