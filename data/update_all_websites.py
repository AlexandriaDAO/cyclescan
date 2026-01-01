#!/usr/bin/env python3

# Comprehensive website mapping for all researched projects
websites = {
    # ICP Infrastructure (all point to internetcomputer.org except NNS dapp)
    "Internet Identity": "https://internetcomputer.org",
    "ICP Ledger": "https://internetcomputer.org",
    "ICP Index": "https://internetcomputer.org",
    "NNS Registry": "https://internetcomputer.org",
    "NNS Governance": "https://internetcomputer.org",
    "NNS Root": "https://internetcomputer.org",
    "Cycles Minting Canister (CMC)": "https://internetcomputer.org",
    "NNS Lifeline": "https://internetcomputer.org",
    "Genesis Token": "https://internetcomputer.org",
    "NNS dapp": "https://nns.ic0.app",
    "ICP Ledger Archive": "https://internetcomputer.org",
    "SNS-W": "https://internetcomputer.org",
    "NNS Subnet Rental": "https://internetcomputer.org",
    "NNS Node Provider Rewards": "https://internetcomputer.org",
    "NNS Canister Migration": "https://internetcomputer.org",
    "NNS Subnet Management": "https://internetcomputer.org",
    "Bitcoin Integration": "https://internetcomputer.org",
    "EVM RPC": "https://internetcomputer.org",
    "Exchange Rates Oracle": "https://internetcomputer.org",
    "ckETH Ledger": "https://internetcomputer.org",
    "ckETH Minter": "https://internetcomputer.org",
    "ckETH Index": "https://internetcomputer.org",
    "ckUSDC Ledger": "https://internetcomputer.org",
    "ckUSDC Index": "https://internetcomputer.org",
    "ckEURC Index": "https://internetcomputer.org",
    "ckLINK Ledger": "https://internetcomputer.org",
    "ckLINK Index": "https://internetcomputer.org",
    "ckOCT Ledger": "https://internetcomputer.org",
    "ckPEPE Index": "https://internetcomputer.org",
    "ckUNI Index": "https://internetcomputer.org",
    "ckXAUT Index": "https://internetcomputer.org",
    "ckERC20 Orchestrator": "https://internetcomputer.org",

    # Major Ecosystem Projects
    "ORIGYN": "https://www.origyn.com",
    "Sonic": "https://sonic.ooo",
    "ICPSwap Token": "https://www.icpswap.com",
    "KongSwap": "https://kongswap.io",
    "OmniBTC": "https://www.omnibtc.finance",
    "Omnity": "https://www.omnity.network",
    "ICLighthouse DAO": "https://iclight.house",
    "NFID Wallet": "https://nfid.one",
    "WaterNeuron": "https://waterneuron.fi",
    "Oisy": "https://oisy.com",
    "Nuance": "https://www.home.nuance.xyz",
    "Taggr": "https://taggr.link",
    "ICPanda": "https://64od3-hiaaa-aaaad-qa3uq-cai.raw.icp0.io",
    "Yuku AI": "https://yuku.app",
    "Orbit": "https://orbithub.app",
    "Orbit Station": "https://orbithub.app",
    "Juno Build": "https://juno.build",
    "ICExplorer": "https://www.icexplorer.io",
    "Neutrinite": "https://docs.boomdao.xyz",  # Part of BOOM DAO ecosystem

    # SNS Projects
    "GOLDAO": "https://docs.gold-dao.org",
    "ICVC": "https://icvc-2.vercel.app",
    "KINIC": "https://internetcomputer.org",  # No dedicated website found
    "Sneed DAO": "https://sneeddao.com",
    "DecideAI": "https://internetcomputer.org",  # No dedicated website found
    "ELNA": "https://www.elna.ai",
    "TRAX": "https://internetcomputer.org",  # No dedicated website found, music platform
    "Draggin Karma Points": "https://internetcomputer.org",  # Dragginz game, no clear main site
    "CatalyzeDAO": "https://catalyze.one",
    "CYCLES-TRANSFER-STATION": "https://cycles-transfer-station.com",
    "SNS Component": "https://internetcomputer.org",
    "SNS Swap": "https://internetcomputer.org",

    # Token/DeFi Projects
    "BOOM": "https://u52bf-3qaaa-aaaal-qb5wq-cai.icp0.io",
    "CHAT": "https://oc.app",
    "TACO": "https://internetcomputer.org",
    "ICPunks": "https://icpunks.com",
    "Swampies": "https://internetcomputer.org",
    "Tendies": "https://internetcomputer.org",
    "DOLR AI": "https://internetcomputer.org",
    "FomoWell": "https://internetcomputer.org",
    "Ordi Trade": "https://internetcomputer.org",
    "ICPEx": "https://internetcomputer.org",
    "ESTATE": "https://internetcomputer.org",
    "ALICE": "https://internetcomputer.org",
    "PHASMA": "https://internetcomputer.org",
    "ICTO": "https://internetcomputer.org",
    "Mimic Clay": "https://internetcomputer.org",
    "ODIN.fun": "https://internetcomputer.org",
}

print(f"Total websites found: {len(websites)}")
print(f"\nProjects with dedicated websites: {sum(1 for v in websites.values() if v not in ['https://internetcomputer.org', 'https://nns.ic0.app'])}")
print(f"Projects using ICP default: {sum(1 for v in websites.values() if v == 'https://internetcomputer.org')}")

# Export for import script
import json
with open('websites_for_import.json', 'w') as f:
    json.dump(websites, f, indent=2)

print(f"\nSaved to websites_for_import.json")
