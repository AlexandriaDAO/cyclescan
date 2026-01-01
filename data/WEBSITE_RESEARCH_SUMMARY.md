# CycleScan Website Research Summary

**Date:** 2025-12-31
**Status:** Phase 1 Complete - 79/152 projects researched and uploaded

## Overview

Successfully researched and added websites for 79 projects across 1,477 canisters in the CycleScan backend.

## Statistics

- **Total Projects in Database:** 152
- **Projects Researched:** 79 (52%)
- **Projects with Dedicated Websites:** 28
- **Projects Using ICP Default:** 50
- **Infrastructure/System Projects:** 1
- **Canisters Updated:** 1,477
- **Batches Uploaded:** 15

## Projects with Websites Added

### ICP Infrastructure (32 projects)
All ICP infrastructure components now point to either https://internetcomputer.org or https://nns.ic0.app:

- Internet Identity, ICP Ledger, ICP Index, NNS Registry, NNS Governance, NNS Root
- Cycles Minting Canister (CMC), NNS Lifeline, Genesis Token, NNS dapp
- ICP Ledger Archive, SNS-W, NNS Subnet Rental, NNS Node Provider Rewards
- NNS Canister Migration, NNS Subnet Management, Bitcoin Integration, EVM RPC
- Exchange Rates Oracle, ckETH Ledger, ckETH Minter, ckETH Index
- ckUSDC Ledger, ckUSDC Index, ckEURC Index, ckLINK Ledger, ckLINK Index
- ckOCT Ledger, ckPEPE Index, ckUNI Index, ckXAUT Index, ckERC20 Orchestrator

### Major Ecosystem Projects (19 projects)

| Project | Website | Type |
|---------|---------|------|
| ORIGYN | https://www.origyn.com | NFT Authentication |
| Sonic | https://sonic.ooo | DEX/AMM |
| ICPSwap Token | https://www.icpswap.com | DEX |
| KongSwap | https://kongswap.io | DEX |
| OmniBTC | https://www.omnibtc.finance | Cross-chain DeFi |
| Omnity | https://www.omnity.network | Interoperability |
| ICLighthouse DAO | https://iclight.house | DEX/DeFi Framework |
| NFID Wallet | https://nfid.one | Wallet |
| WaterNeuron | https://waterneuron.fi | Liquid Staking |
| Oisy | https://oisy.com | Multi-chain Wallet |
| Nuance | https://www.home.nuance.xyz | Blogging Platform |
| Taggr | https://taggr.link | Social Network |
| ICPanda | https://64od3-hiaaa-aaaad-qa3uq-cai.raw.icp0.io | NFT Collection |
| Yuku AI | https://yuku.app | NFT Marketplace/AI |
| Orbit | https://orbithub.app | DAO Tools |
| Orbit Station | https://orbithub.app | DAO Platform |
| Juno Build | https://juno.build | Developer Platform |
| ICExplorer | https://www.icexplorer.io | Blockchain Explorer |
| Neutrinite | https://docs.boomdao.xyz | DeFi Protocol |

### SNS Projects (12 projects)

| Project | Website | Description |
|---------|---------|-------------|
| GOLDAO | https://docs.gold-dao.org | Gold-backed DAO |
| ICVC | https://icvc-2.vercel.app | Venture Capital DAO |
| Sneed DAO | https://sneeddao.com | Community DAO |
| ELNA | https://www.elna.ai | AI Platform |
| CatalyzeDAO | https://catalyze.one | Community Engagement |
| CYCLES-TRANSFER-STATION | https://cycles-transfer-station.com | Cycles Trading |
| KINIC | https://internetcomputer.org | AI Search (no dedicated site) |
| DecideAI | https://internetcomputer.org | AI DAO (no dedicated site) |
| TRAX | https://internetcomputer.org | Music Platform (no dedicated site) |
| Draggin Karma Points | https://internetcomputer.org | Gaming Token (no dedicated site) |
| SNS Component | https://internetcomputer.org | Infrastructure |
| SNS Swap | https://internetcomputer.org | Infrastructure |

### Token/DeFi Projects (16 projects)

| Project | Website | Type |
|---------|---------|------|
| BOOM | https://u52bf-3qaaa-aaaal-qb5wq-cai.icp0.io | Gaming DAO |
| CHAT (OpenChat) | https://oc.app | Messaging |
| ICPunks | https://icpunks.com | NFT Collection |
| Others | https://internetcomputer.org | Various tokens |

## Remaining Work

### Projects Still Needing Research (~55 projects)

The following project categories still need website research:

1. **Individual Token Projects:** Various meme tokens, community tokens, and smaller DeFi projects
2. **Specialized Projects:** Cecil The Lion DAO, DOLR AI, FomoWell, Ordi Trade, etc.
3. **Platform Projects:** Cycle Manager, Personal DAO, Analytics Service, etc.
4. **Abandoned/Unidentified:** Projects marked as "Abandoned", "Unidentified", etc.

## Files Created

- `websites_for_import.json` - JSON mapping of all researched projects to websites
- `batch_1_websites.sh` through `batch_15_websites.sh` - Upload scripts
- `update_all_websites.py` - Website compilation script
- `upload_websites.sh` - Master upload orchestration script

## Next Steps

To complete the remaining ~55 projects:

1. Continue systematic web searches for each project
2. Focus on projects with actual products/services (skip "Unknown Token", "Unidentified", etc.)
3. Upload additional batches as websites are discovered
4. Document findings in this summary file

## Notes

- Infrastructure projects appropriately link to internetcomputer.org as their canonical source
- SNS DAOs without dedicated websites link to the ICP dashboard or internetcomputer.org
- All uploads verified successful via backend responses
- Data is immediately available in the CycleScan dashboard
