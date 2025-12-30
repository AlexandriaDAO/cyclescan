#!/usr/bin/env python3
"""
Fetch all SNS canisters and generate import data.
Output: sns_canisters.json - ready for import_canisters()

Usage: python3 fetch_sns.py
"""

import subprocess
import json
import re
import sys

SNS_WASM_CANISTER = "qaa6y-5yaaa-aaaaa-aaafa-cai"

def run_dfx(canister_id, method, args="(record {})"):
    """Run dfx canister call and return output."""
    cmd = ["dfx", "canister", "--network", "ic", "call", canister_id, method, args]
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        return None
    return result.stdout

def extract_principals(text, pattern):
    """Extract principal IDs matching a pattern."""
    matches = re.findall(rf'{pattern} = opt principal "([a-z0-9-]+-cai)"', text)
    return matches

def main():
    print("Fetching list of deployed SNSes...")

    # Get all SNS instances
    raw = run_dfx(SNS_WASM_CANISTER, "list_deployed_snses")
    if not raw:
        print("ERROR: Failed to fetch SNS list")
        sys.exit(1)

    # Extract root canister IDs
    roots = extract_principals(raw, "root_canister_id")
    print(f"Found {len(roots)} SNSes")

    canisters = []

    for i, sns_root in enumerate(roots):
        print(f"  [{i+1}/{len(roots)}] Fetching {sns_root}...")

        summary = run_dfx(sns_root, "get_sns_canisters_summary")
        if not summary:
            print(f"    FAILED - skipping")
            continue

        # Extract ALL canister IDs from the response
        # The response contains canister_id = opt principal "xxx-cai" for each canister
        canister_ids = re.findall(r'canister_id = opt principal "([a-z0-9-]+)"', summary)

        # Deduplicate and add all found canisters
        seen = set()
        for cid in canister_ids:
            if cid not in seen:
                seen.add(cid)
                canisters.append({
                    "canister_id": cid,
                    "proxy_id": sns_root,
                    "proxy_type": "SnsRoot"
                })

        print(f"    Found {len(seen)} canisters")

    # Write output
    output_file = "sns_canisters.json"
    with open(output_file, "w") as f:
        json.dump(canisters, f, indent=2)

    print(f"\n=== Results ===")
    print(f"SNSes processed: {len(roots)}")
    print(f"Canisters found: {len(canisters)}")
    print(f"Output: {output_file}")

    # Show breakdown
    by_root = {}
    for c in canisters:
        root = c["proxy_id"]
        by_root[root] = by_root.get(root, 0) + 1

    print(f"\nPer-SNS breakdown (top 10):")
    for root, count in sorted(by_root.items(), key=lambda x: -x[1])[:10]:
        print(f"  {root}: {count} canisters")

if __name__ == "__main__":
    main()
