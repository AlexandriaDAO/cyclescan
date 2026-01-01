#!/usr/bin/env python3

# Categorize projects for systematic website research

# Skip these - they're generic or unidentifiable
SKIP_PROJECTS = {
    "Unidentified",
    "Unidentified (no candid)",
    "Unidentified (deleted)",
    "Unidentified (empty service)",
    "Unidentified (minimal interface)",
    "Unknown",
    "Unknown Token",
    "Asset Canister",
    "Logger Canister",
    "Canister Upgrader",
    "Cycles Wallet",
    "Canister Monitor",
    "Canister Status Query",
    "Motoko",
    "ICRC Archive",
    "ICRC-1 Archive",
    "ICRC-3 Archive",
}

# Official ICP/Dfinity infrastructure
ICP_INFRASTRUCTURE = [
    "Internet Identity",
    "NNS Registry",
    "NNS Governance",
    "ICP Ledger",
    "NNS Root",
    "Cycles Minting Canister (CMC)",
    "NNS Lifeline",
    "Genesis Token",
    "NNS dapp",
    "ICP Ledger Archive",
    "SNS-W",
    "ICP Index",
    "NNS Subnet Rental",
    "NNS Node Provider Rewards",
    "NNS Canister Migration",
    "NNS Subnet Management",
    "Bitcoin Integration",
    "EVM RPC",
    "Exchange Rates Oracle",
    "ckETH Ledger",
    "ckETH Minter",
    "ckETH Index",
    "ckUSDC Ledger",
    "ckUSDC Index",
    "ckEURC Index",
    "ckLINK Ledger",
    "ckLINK Index",
    "ckOCT Ledger",
    "ckPEPE Index",
    "ckUNI Index",
    "ckXAUT Index",
    "ckERC20 Orchestrator",
]

# Major ICP ecosystem projects (high priority)
MAJOR_ECOSYSTEM = [
    "ORIGYN",
    "Sonic",
    "ICPSwap Token",
    "KongSwap",
    "OmniBTC",
    "Omnity",
    "ICLighthouse DAO",
    "NFID Wallet",
    "WaterNeuron",
    "Oisy",
    "OpenChat",  # from CHAT
    "Nuance",
    "Taggr",
    "ICPanda",
    "Yuku AI",
    "Orbit",
    "Orbit Station",
    "Juno Build",
    "ICExplorer",
    "Neutrinite",
]

# SNS Projects
SNS_PROJECTS = [
    "GOLDAO",
    "ICVC",
    "KINIC",
    "Sneed DAO",
    "DecideAI",
    "ELNA",
    "TRAX",
    "Draggin Karma Points",
    "CatalyzeDAO",
    "CYCLES-TRANSFER-STATION",
    "SNS Component",
    "SNS Swap",
]

# Token/DeFi projects
TOKEN_PROJECTS = [
    "BOOM",
    "CHAT",
    "TACO",
    "ICPunks",
    "Swampies",
    "Tendies",
    "DOLR AI",
    "FomoWell",
    "Ordi Trade",
    "ICPEx",
    "ESTATE",
    "ALICE",
    "PHASMA",
    "ICTO",
    "Mimic Clay",
    "ODIN.fun",
]

with open('unique_projects.txt', 'r') as f:
    all_projects = [line.strip() for line in f if line.strip()]

# Filter projects to research
projects_to_research = []
skipped = []

for project in all_projects:
    if project in SKIP_PROJECTS or project.startswith("Orbit Upgrader"):
        skipped.append(project)
    else:
        projects_to_research.append(project)

print(f"Total projects: {len(all_projects)}")
print(f"Projects to research: {len(projects_to_research)}")
print(f"Skipped (generic/unidentified): {len(skipped)}")
print()

# Categorize for research priority
print("=" * 80)
print("PRIORITY 1: ICP Infrastructure")
print("=" * 80)
for p in ICP_INFRASTRUCTURE:
    if p in projects_to_research:
        print(f"  - {p}")

print()
print("=" * 80)
print("PRIORITY 2: Major Ecosystem Projects")
print("=" * 80)
for p in MAJOR_ECOSYSTEM:
    if p in projects_to_research:
        print(f"  - {p}")

print()
print("=" * 80)
print("PRIORITY 3: SNS Projects")
print("=" * 80)
for p in SNS_PROJECTS:
    if p in projects_to_research:
        print(f"  - {p}")

print()
print("=" * 80)
print("PRIORITY 4: Token/DeFi Projects")
print("=" * 80)
for p in TOKEN_PROJECTS:
    if p in projects_to_research:
        print(f"  - {p}")

# Save research list
with open('projects_to_research.txt', 'w') as f:
    f.write("# Projects to research for websites\n\n")
    f.write("## ICP Infrastructure\n")
    for p in ICP_INFRASTRUCTURE:
        if p in projects_to_research:
            f.write(f"{p}\n")

    f.write("\n## Major Ecosystem\n")
    for p in MAJOR_ECOSYSTEM:
        if p in projects_to_research:
            f.write(f"{p}\n")

    f.write("\n## SNS Projects\n")
    for p in SNS_PROJECTS:
        if p in projects_to_research:
            f.write(f"{p}\n")

    f.write("\n## Token/DeFi Projects\n")
    for p in TOKEN_PROJECTS:
        if p in projects_to_research:
            f.write(f"{p}\n")

    f.write("\n## Other Projects\n")
    other_projects = set(projects_to_research) - set(ICP_INFRASTRUCTURE) - set(MAJOR_ECOSYSTEM) - set(SNS_PROJECTS) - set(TOKEN_PROJECTS)
    for p in sorted(other_projects):
        f.write(f"{p}\n")

print("\nSaved research list to projects_to_research.txt")
