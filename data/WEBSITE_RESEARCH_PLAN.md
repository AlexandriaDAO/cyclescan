# CycleScan Website Research - Complete Plan

**Objective:** Find and verify the correct official website for every project in the CycleScan database, ensuring accuracy through proper research and verification.

## Current Situation

**Database:** 152 unique projects across 3,140+ canisters
**Status:** Previous attempt uploaded ~79 projects, but many websites are INCORRECT or unverified
**Problem:** Websites were not properly researched - some were guessed, some defaulted to internetcomputer.org incorrectly, and verification was skipped

### Known Errors from Previous Attempt
- **Orbit**: Listed as `https://orbithub.app/` (doesn't exist) → Should be `https://orbit.global`
- **ODIN.fun**: Listed as `https://internetcomputer.org` → Should be `https://odin.fun`
- Many others likely incorrect or unverified

## Research Methodology - DO THIS RIGHT

### 1. Research Standards

For EACH project, you MUST:

1. **Search with multiple queries** - Don't rely on a single search
   - "[Project Name] Internet Computer official website"
   - "[Project Name] ICP website"
   - "[Project Name] official site"
   - Check project Twitter/X, GitHub, official forums

2. **Verify the website actually exists**
   - Use WebFetch to confirm the URL loads
   - Check it's the actual project, not a placeholder
   - Look for project branding, correct name, ICP references

3. **Prefer official domains over hosting platforms**
   - Example: `orbit.global` > `orbithub.app`
   - Look for custom domains, not just `.icp0.io` or `.ic0.app` unless that's the only option

4. **Check multiple sources for confirmation**
   - Project's own GitHub repo README
   - ICP Dashboard project info
   - Official announcements on forum.dfinity.org
   - CoinGecko/CoinMarketCap for token projects

5. **Document your findings**
   - Note the source where you found each website
   - Flag any uncertainties for manual review

### 2. Categories and Special Handling

#### ICP Infrastructure Projects
**These are legitimate internetcomputer.org projects:**
- Internet Identity, NNS components, ICP Ledger, CMC, Bitcoin Integration, EVM RPC
- All ck-tokens (ckBTC, ckETH, ckUSDC, etc.) - chain-key wrapped assets
- Official Dfinity/ICP Foundation projects

**Verification:** Check dashboard.internetcomputer.org for official canister IDs

#### SNS DAO Projects
**Research approach:**
1. Check the SNS DAO dashboard for project links
2. Look for official documentation/whitepaper
3. Search "[Project] SNS DAO" for official announcements
4. Many SNS projects have dedicated websites - find them!

**Common patterns:**
- Project names often have .com, .io, .xyz domains
- Check social media links in SNS proposals
- Look for "official website" in forum posts about the SNS launch

#### Token Projects
**For tokens (especially meme tokens):**
- Search "[Token Symbol] [Token Name] ICP website"
- Check ICPCoins.com, ICPSwap, Kong listings for links
- Look on X/Twitter for official project accounts
- Many have simple sites: `[name].com`, `[name].io`, `ic[name].com`

#### Platform Projects (DEXs, Wallets, etc.)
**These MUST have websites - search thoroughly:**
- DEXs: Sonic, ICPSwap, KongSwap, ICLighthouse, etc.
- Wallets: NFID, Oisy, etc.
- Infrastructure: Juno, Orbit, ICExplorer, etc.

**Verification:**
- The website should be a functioning dApp or landing page
- Should have clear ICP/IC branding or mentions
- Should match the project's stated purpose

### 3. When a Website Truly Doesn't Exist

Only mark as "no website" if:
- Extensive searching yields nothing
- Project is clearly abandoned (check last activity)
- Project is infrastructure-only (some canisters/archives)
- It's labeled "Unidentified" or "Unknown Token" with no metadata

**For these cases:**
- Use `null` (no website field) instead of defaulting to internetcomputer.org
- Or use the ICP Dashboard canister URL as last resort
- Document why no website was found

## Complete Project List to Research

### Priority 1: Fix Known Errors (URGENT)
1. Orbit - Currently wrong
2. ODIN.fun - Currently wrong
3. Review ALL 79 previously uploaded projects for accuracy

### Priority 2: Major Projects Without Websites Yet

**SNS DAOs to research:**
- Cecil The Lion DAO
- Personal DAO
- ICFC
- ICPunks (verify current website is correct)
- Squirrel (SNS Aggregator)

**Token/DeFi Projects:**
- TACO
- Tendies
- Swampies
- FomoWell
- Ordi Trade
- ICPEx
- ESTATE
- ALICE
- PHASMA
- ICTO
- Mimic Clay
- DOLR AI

**Platform Projects:**
- Cycle Manager
- CanDB Storage
- Exchange Registry
- DEX
- Discord Bot
- Forum/Board
- Social Platform
- Social/Forum App
- NFT Runes Exchange
- Bitcoin Runes Etching
- Bills Payment App
- Analytics Service

**Token Projects (many likely small/meme tokens):**
- 5000 SLICES Token
- Alice Token
- Aptos Token
- Avocado Research Token
- BOB Token
- Bitcoin Token
- BoB (Burn or Burn)
- Butten Bun Token
- Cardano Token
- GoldFish Token
- GoldSlice Token
- GRAVE Token
- Granny VS Internet Token
- Hell O Token
- Kaspa Token
- RONJU Token
- Sachin Tendulkar Token
- The Needful DO Token
- ToKeN Token
- Trump Token
- TRON Token
- TST15 Token
- WaterMelon Token
- Windoge98 Token
- XRP Token
- polkadot Token
- vyt Token
- OHSHII
- Blackhole (ninegua)
- TipJar

**Other Projects:**
- CANI DAO
- CLOUD Index
- ICRC Index (multiple instances - check if system component or project)
- ICRC Token
- NFT (generic - may be system component)
- OHSHII Index
- RON Index
- SHOW Index
- CBOW Index (if exists)

### Priority 3: Verify Generic/Infrastructure
These may legitimately be infrastructure with no dedicated website:
- Canister Status Query
- Logger Canister
- Canister Monitor
- Canister Upgrader
- Cycle Manager
- ICRC Archive instances
- Various Index canisters

**Research:** Check if these are actual projects or just utility canisters

## Step-by-Step Execution Plan

### Phase 1: Audit and Fix (Days 1-2)
1. Export current websites from backend
2. Verify each of the 79 previously uploaded websites
3. Create corrections list
4. Fix all errors immediately

### Phase 2: Research Remaining Projects (Days 3-5)
1. Work through Priority 2 list systematically
2. For each project:
   - Multiple web searches
   - Verify website loads (WebFetch)
   - Document source
   - Confirm it's the right project
3. Batch upload verified websites every 20-30 projects

### Phase 3: Handle Edge Cases (Day 6)
1. Research Priority 3 infrastructure items
2. Make final determinations on "no website" projects
3. Document all "no website" decisions with reasoning

### Phase 4: Final Verification (Day 7)
1. Random sample check 25% of all websites
2. Verify all high-profile projects (SNS DAOs, major DEXs, etc.)
3. Create final report with confidence levels

## Quality Assurance Checklist

For each website entry, ensure:
- [ ] Website URL is valid and loads
- [ ] Website matches the project name and type
- [ ] Website shows ICP/Internet Computer affiliation
- [ ] URL is the official/canonical domain (not a mirror or test site)
- [ ] Source documented (where you found it)
- [ ] Cross-referenced with at least 2 sources
- [ ] For tokens: Verified on a DEX or token listing site
- [ ] For SNS DAOs: Verified against SNS dashboard or forum announcement

## Output Format

Create a JSON file: `verified_websites.json`

```json
{
  "Project Name": {
    "website": "https://example.com",
    "verified": true,
    "sources": [
      "GitHub repo README",
      "SNS Dashboard",
      "forum.dfinity.org announcement"
    ],
    "confidence": "high",
    "notes": "Official domain confirmed from multiple sources"
  },
  "Another Project": {
    "website": null,
    "verified": true,
    "sources": ["Extensive search"],
    "confidence": "high",
    "notes": "No website found - project appears abandoned since 2022"
  }
}
```

## Success Criteria

- **100% of projects researched** - every project has either a verified website or documented "no website"
- **High confidence** - >90% of websites marked "high confidence"
- **All major projects correct** - SNS DAOs, DEXs, wallets, infrastructure all 100% accurate
- **Documented sources** - every website has source attribution
- **Verified functionality** - random sample shows websites load correctly

## Tools Available

- **WebSearch** - For finding project information
- **WebFetch** - For verifying websites load and checking content
- **Bash/dfx** - For querying ICP dashboard and backend
- **Read/Write** - For managing data files

## Anti-Patterns to Avoid

❌ **DON'T:**
- Guess website URLs
- Default to internetcomputer.org unless it's actually an ICP infrastructure project
- Use the first search result without verification
- Assume domain patterns (projectname.com) without checking
- Skip verification steps
- Rely on a single source

✅ **DO:**
- Search multiple ways for each project
- Verify every URL loads
- Cross-reference multiple sources
- Document your research
- Ask for clarification when uncertain
- Use WebFetch to verify website content
- Check official ICP channels (forum, dashboard, GitHub)

## Reference Resources

- **ICP Dashboard:** https://dashboard.internetcomputer.org
- **ICP Forum:** https://forum.dfinity.org
- **SNS Launchpad:** SNS proposals and documentation
- **ICPCoins:** Token listings and info
- **ICPSwap/Kong:** DEX listings with project links
- **GitHub:** Search for "[project-name] internet-computer"

## Notes for Agent Execution

- This is a large task - expect 20-40 hours of research time
- Prioritize accuracy over speed
- Batch uploads every 20-30 verified websites
- Keep detailed notes for the final report
- Flag any uncertainties for human review
- Update progress regularly in a tracking document

## Final Deliverables

1. `verified_websites.json` - Complete mapping with sources and confidence
2. `website_corrections.json` - List of corrections from previous attempt
3. `research_report.md` - Summary of findings, confidence levels, issues
4. Upload scripts for backend updates
5. This plan with execution notes and lessons learned
