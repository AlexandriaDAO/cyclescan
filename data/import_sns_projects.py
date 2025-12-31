#!/usr/bin/env python3
"""
Import SNS project names into research_results.json

This script:
1. Reads sns_canisters.json to get all SNS canisters grouped by proxy_id (SNS root)
2. Queries icrc1_name() on each SNS ledger to get the project name
3. Creates research_results entries for each SNS canister
4. Appends to research_results.json
"""

import json
import subprocess
import re
from collections import defaultdict

# SNS Registry data - root_canister_id -> ledger_canister_id
# Extracted from list_deployed_snses output
SNS_REGISTRY = {
    "zxeu2-7aaaa-aaaaq-aaafa-cai": "zfcdd-tqaaa-aaaaq-aaaga-cai",
    "3e3x2-xyaaa-aaaaq-aaalq-cai": "2ouva-viaaa-aaaaq-aaamq-cai",
    "23ten-uaaaa-aaaaq-aaapa-cai": "7ajy4-sqaaa-aaaaq-aaaqa-cai",
    "7jkta-eyaaa-aaaaq-aaarq-cai": "73mez-iiaaa-aaaaq-aaasq-cai",
    "67bll-riaaa-aaaaq-aaauq-cai": "6rdgd-kyaaa-aaaaq-aaavq-cai",
    "6kg2g-qaaaa-aaaaq-aaaxa-cai": "4q2s2-oqaaa-aaaaq-aaaya-cai",
    "4m6il-zqaaa-aaaaq-aaa2a-cai": "4c4fd-caaaa-aaaaq-aaa3a-cai",
    "5psbn-niaaa-aaaaq-aaa4q-cai": "5bqmf-wyaaa-aaaaq-aaa5q-cai",
    "55uwu-byaaa-aaaaq-aaa7q-cai": "wedc6-xiaaa-aaaaq-aabaq-cai",
    "w7g63-nqaaa-aaaaq-aabca-cai": "wrett-waaaa-aaaaq-aabda-cai",
    "x4kx5-ziaaa-aaaaq-aabeq-cai": "xsi2v-cyaaa-aaaaq-aabfq-cai",
    "xjngq-yaaaa-aaaaq-aabha-cai": "vtrom-gqaaa-aaaaq-aabia-cai",
    "v2sfq-qyaaa-aaaaq-aabjq-cai": "viusj-4iaaa-aaaaq-aabkq-cai",
    "uly3p-iqaaa-aaaaq-aabma-cai": "uf2wh-taaaa-aaaaq-aabna-cai",
    "u67kc-jyaaa-aaaaq-aabpq-cai": "rffwt-piaaa-aaaaq-aabqq-cai",
    "rzbmc-yiaaa-aaaaq-aabsq-cai": "rxdbk-dyaaa-aaaaq-aabtq-cai",
    "qtooy-2yaaa-aaaaq-aabvq-cai": "qbizb-wiaaa-aaaaq-aabwq-cai",
    "s4vxj-faaaa-aaaaq-aabza-cai": "sotaq-jqaaa-aaaaq-aab2a-cai",
    "shqlm-7yaaa-aaaaq-aab3q-cai": "tn7jw-5iaaa-aaaaq-aab4q-cai",
    "tw2vt-hqaaa-aaaaq-aab6a-cai": "tyyy3-4aaaa-aaaaq-aab7a-cai",
    "ecu3s-hiaaa-aaaaq-aacaq-cai": "emww2-4yaaa-aaaaq-aacbq-cai",
    "extk7-gaaaa-aaaaq-aacda-cai": "f54if-eqaaa-aaaaq-aacea-cai",
    "fp274-iaaaa-aaaaq-aacha-cai": "hvgxa-wqaaa-aaaaq-aacia-cai",
    "hjcnr-bqaaa-aaaaq-aacka-cai": "hhaaz-2aaaa-aaaaq-aacla-cai",
    "gkoex-viaaa-aaaaq-aacmq-cai": "gemj7-oyaaa-aaaaq-aacnq-cai",
    "gyito-zyaaa-aaaaq-aacpq-cai": "ddsp7-7iaaa-aaaaq-aacqq-cai",
    "d7wvo-iiaaa-aaaaq-aacsq-cai": "druyg-tyaaa-aaaaq-aactq-cai",
    "csyra-haaaa-aaaaq-aacva-cai": "ca6gz-lqaaa-aaaaq-aacwa-cai",
    "cj5nf-5yaaa-aaaaq-aacxq-cai": "atbfz-diaaa-aaaaq-aacyq-cai",
    "abhsa-pyaaa-aaaaq-aac3q-cai": "bliq2-niaaa-aaaaq-aac4q-cai",
    "bxmkl-2iaaa-aaaaq-aac6q-cai": "bzohd-byaaa-aaaaq-aac7q-cai",
    "ko36b-myaaa-aaaaq-aadbq-cai": "k45jy-aiaaa-aaaaq-aadcq-cai",
    "l7ra6-uqaaa-aaaaq-aadea-cai": "lrtnw-paaaa-aaaaq-aadfa-cai",
    "leu43-oiaaa-aaaaq-aadgq-cai": "lkwrt-vyaaa-aaaaq-aadhq-cai",
    "jmod6-4iaaa-aaaaq-aadkq-cai": "jcmow-hyaaa-aaaaq-aadlq-cai",
    "ibahq-taaaa-aaaaq-aadna-cai": "itgqj-7qaaa-aaaaq-aadoa-cai",
    "nb7he-piaaa-aaaaq-aadqq-cai": "np5km-uyaaa-aaaaq-aadrq-cai",
    "nuywj-oaaaa-aaaaq-aadta-cai": "m6xut-mqaaa-aaaaq-aadua-cai",
    "mctoc-3qaaa-aaaaq-aadwa-cai": "mmrdk-aaaaa-aaaaq-aadxa-cai",
    "ormnc-tiaaa-aaaaq-aadyq-cai": "o7oak-iyaaa-aaaaq-aadzq-cai",
    "pvbcq-kiaaa-aaaaq-aad6q-cai": "p3dpy-ryaaa-aaaaq-aad7q-cai",
    "pnthx-iiaaa-aaaaq-aaeba-cai": "p7vqo-eyaaa-aaaaq-aaeca-cai",
    "pww3s-sqaaa-aaaaq-aaedq-cai": "o4zzi-qaaaa-aaaaq-aaeeq-cai",
    "oh4fn-kyaaa-aaaaq-aaega-cai": "oj6if-riaaa-aaaaq-aaeha-cai",
    "m2blf-zqaaa-aaaaq-aaejq-cai": "mih44-vaaaa-aaaaq-aaekq-cai",
    "nllv2-byaaa-aaaaq-aaema-cai": "nfjys-2iaaa-aaaaq-aaena-cai",
    "n6mex-aqaaa-aaaaq-aaepq-cai": "ifwyg-gaaaa-aaaaq-aaeqq-cai",
    "izscx-raaaa-aaaaq-aaesq-cai": "ixqp7-kqaaa-aaaaq-aaetq-cai",
    "ju4gz-6iaaa-aaaaq-aaeva-cai": "jg2ra-syaaa-aaaaq-aaewa-cai",
    "jpz24-eqaaa-aaaaq-aaexq-cai": "lvfsa-2aaaa-aaaaq-aaeyq-cai",
    "lacdn-3iaaa-aaaaq-aae3a-cai": "kknbx-zyaaa-aaaaq-aae4a-cai",
    "kwj3g-oyaaa-aaaaq-aae6a-cai": "kylwo-viaaa-aaaaq-aae7a-cai",
}

# Also need governance, index, swap for role identification
SNS_FULL_REGISTRY = {}  # Will be populated from the full data


def get_icrc1_name(ledger_id: str) -> str:
    """Query icrc1_name from a ledger canister"""
    try:
        result = subprocess.run(
            ["dfx", "canister", "--network", "ic", "call", ledger_id, "icrc1_name", "()"],
            capture_output=True,
            text=True,
            timeout=30
        )
        if result.returncode == 0:
            # Parse output like: ("Draggin Karma Points")
            match = re.search(r'\("(.+?)"\)', result.stdout)
            if match:
                return match.group(1)
    except Exception as e:
        print(f"  Error querying {ledger_id}: {e}")
    return None


def get_canister_role(canister_id: str, proxy_id: str, sns_data: dict) -> str:
    """Determine the role of an SNS canister"""
    if proxy_id not in sns_data:
        return "SNS canister"

    info = sns_data[proxy_id]
    if canister_id == info.get("root"):
        return "SNS Root canister"
    elif canister_id == info.get("governance"):
        return "SNS Governance canister"
    elif canister_id == info.get("ledger"):
        return "SNS Ledger canister"
    elif canister_id == info.get("index"):
        return "SNS Index canister"
    elif canister_id == info.get("swap"):
        return "SNS Swap canister"
    elif canister_id in info.get("dapps", []):
        return "SNS Dapp canister"
    elif canister_id in info.get("archives", []):
        return "SNS Archive canister"
    else:
        return "SNS canister"


def main():
    # Load SNS canisters
    print("Loading sns_canisters.json...")
    with open("sns_canisters.json", "r") as f:
        sns_canisters = json.load(f)
    print(f"  Loaded {len(sns_canisters)} SNS canisters")

    # Load existing research results
    print("Loading research_results.json...")
    with open("research_results.json", "r") as f:
        research_results = json.load(f)
    print(f"  Loaded {len(research_results)} existing entries")

    # Create set of existing canister IDs
    existing_ids = {r["canister_id"] for r in research_results}

    # Group SNS canisters by proxy_id (root canister)
    sns_groups = defaultdict(list)
    for canister in sns_canisters:
        sns_groups[canister["proxy_id"]].append(canister["canister_id"])
    print(f"  Found {len(sns_groups)} unique SNS roots")

    # Query each SNS ledger for project name
    print("\nQuerying SNS ledgers for project names...")
    project_names = {}
    for root_id, ledger_id in SNS_REGISTRY.items():
        if root_id in sns_groups:
            print(f"  Querying {ledger_id} (root: {root_id})...")
            name = get_icrc1_name(ledger_id)
            if name:
                project_names[root_id] = name
                print(f"    -> {name}")
            else:
                print(f"    -> FAILED")

    print(f"\nSuccessfully got {len(project_names)} project names")

    # Create entries for SNS canisters
    print("\nCreating research entries...")
    new_entries = []
    skipped = 0

    for canister in sns_canisters:
        canister_id = canister["canister_id"]
        proxy_id = canister["proxy_id"]

        # Skip if already in research results
        if canister_id in existing_ids:
            skipped += 1
            continue

        # Get project name from lookup
        project_name = project_names.get(proxy_id, "Unknown SNS")

        # Determine if this is the ledger canister
        is_ledger = SNS_REGISTRY.get(proxy_id) == canister_id

        # Determine role
        role = "SNS Ledger canister" if is_ledger else "SNS canister"
        if canister_id == proxy_id:
            role = "SNS Root canister"

        entry = {
            "canister_id": canister_id,
            "project": project_name,
            "notes": role,
            "is_token": is_ledger
        }

        new_entries.append(entry)

    print(f"  Created {len(new_entries)} new entries")
    print(f"  Skipped {skipped} (already in research_results)")

    # Append to research results
    research_results.extend(new_entries)

    # Save
    print(f"\nSaving to research_results.json ({len(research_results)} total entries)...")
    with open("research_results.json", "w") as f:
        json.dump(research_results, f, indent=2)

    print("\nDone!")

    # Print summary by project
    print("\nSNS Projects Summary:")
    for root_id, name in sorted(project_names.items(), key=lambda x: x[1]):
        count = len(sns_groups.get(root_id, []))
        print(f"  {name}: {count} canisters")


if __name__ == "__main__":
    main()
