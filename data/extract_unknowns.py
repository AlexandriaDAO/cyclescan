#!/usr/bin/env python3
"""Extract all canister IDs with project = null from leaderboard."""

import re
import json
import sys

def parse_leaderboard(filename):
    """Parse the Candid leaderboard output and extract unknown canisters."""
    with open(filename, 'r') as f:
        content = f.read()

    # Find all record entries
    record_pattern = r'record\s*\{[^}]+\}'
    records = re.findall(record_pattern, content, re.DOTALL)

    unknowns = []
    for record in records:
        # Check if this record has project = null
        if 'project = null' in record:
            # Extract canister_id
            canister_match = re.search(r'canister_id = principal "([^"]+)"', record)
            # Extract burn_24h if available
            burn_24h_match = re.search(r'burn_24h = opt \(([0-9_]+) : nat\)', record)
            # Extract burn_7d if available
            burn_7d_match = re.search(r'burn_7d = opt \(([0-9_]+) : nat\)', record)
            # Extract balance
            balance_match = re.search(r'balance = ([0-9_]+) : nat', record)

            if canister_match:
                canister_id = canister_match.group(1)
                burn_24h = int(burn_24h_match.group(1).replace('_', '')) if burn_24h_match else 0
                burn_7d = int(burn_7d_match.group(1).replace('_', '')) if burn_7d_match else 0
                balance = int(balance_match.group(1).replace('_', '')) if balance_match else 0

                unknowns.append({
                    'canister_id': canister_id,
                    'burn_24h': burn_24h,
                    'burn_7d': burn_7d,
                    'balance': balance
                })

    # Sort by burn_24h descending
    unknowns.sort(key=lambda x: x['burn_24h'], reverse=True)

    return unknowns

if __name__ == '__main__':
    unknowns = parse_leaderboard('/tmp/leaderboard_full.txt')

    print(f"Found {len(unknowns)} canisters with project = null")
    print(f"\nTop 10 by 24h burn:")
    for i, canister in enumerate(unknowns[:10], 1):
        print(f"{i}. {canister['canister_id']}: {canister['burn_24h']:,} cycles/24h")

    # Save to file
    output_file = '/home/theseus/alexandria/cyclescan/data/unknowns_to_research.json'
    with open(output_file, 'w') as f:
        json.dump(unknowns, f, indent=2)

    print(f"\nSaved all unknowns to {output_file}")
