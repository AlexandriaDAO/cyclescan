#!/usr/bin/env python3
"""
Second pass: Identify canisters based on Candid patterns and web research.
Processes unknowns that the automated script couldn't identify.
"""

import json
import subprocess
import re
from datetime import datetime

BACKEND = "vohji-riaaa-aaaac-babxq-cai"
RESULTS_IDENTIFIED = "/home/theseus/alexandria/cyclescan/data/research_results2.json"
RESULTS_UNKNOWN = "/home/theseus/alexandria/cyclescan/data/research_results_unknown.json"

CANDID_PATTERNS = {
    # Asset canisters (frontend hosting)
    'asset_canister': {
        'patterns': ['CreateAssetArguments', 'SetAssetContentArguments', 'ChunkId', 'BatchId'],
        'project': 'Asset Canister',
        'category': 'infrastructure',
        'notes': 'Frontend asset canister - website hosting'
    },
    # Orbit Station (multi-sig wallet)
    'orbit_station': {
        'patterns': ['AssetSymbol', 'NetworkId', 'UUID', 'RequestPolicy'],
        'project': 'Orbit Station',
        'category': 'infrastructure',
        'notes': 'Orbit multi-sig wallet/station'
    },
    # ICRC-1 Archive
    'icrc1_archive': {
        'patterns': ['GetTransactionsRequest', 'GetTransactionsResponse', 'ArchivedTransactionResponse', 'QueryArchiveFn'],
        'project': 'ICRC-1 Archive',
        'category': 'infrastructure',
        'notes': 'ICRC-1 token ledger archive'
    },
    # ICPSwap DEX
    'icpswap': {
        'patterns': ['SwapArgs', 'addLiquidity', 'removeLiquidity', 'quote', 'token0', 'token1'],
        'project': 'ICPSwap',
        'category': 'defi',
        'notes': 'ICPSwap AMM pool'
    },
    # Sonic DEX
    'sonic': {
        'patterns': ['addLiquidity', 'swap', 'deposit', 'withdraw', 'getTokenBalance'],
        'project': 'Sonic',
        'category': 'defi',
        'notes': 'Sonic DEX pool or component'
    },
    # NFT canisters
    'nft_dip721': {
        'patterns': ['TokenMetadata', 'ownerOf', 'transferFrom', 'balanceOf', 'totalSupply', 'dip721'],
        'project': 'NFT Collection',
        'category': 'nft',
        'notes': 'DIP-721 NFT collection'
    },
    # HTTP Gateway
    'http_gateway': {
        'patterns': ['http_request', 'HttpRequest', 'HttpResponse', 'streaming_callback'],
        'project': 'HTTP Gateway',
        'category': 'infrastructure',
        'notes': 'HTTP gateway canister'
    },
}

def load_results(filename):
    """Load results file."""
    try:
        with open(filename, 'r') as f:
            return json.load(f)
    except:
        return []

def save_results(filename, results):
    """Save results file."""
    with open(filename, 'w') as f:
        json.dump(results, f, indent=2)

def analyze_candid(candid_summary):
    """Analyze Candid to identify canister type."""
    if not candid_summary:
        return None, None, None

    for pattern_name, pattern_info in CANDID_PATTERNS.items():
        # Check if all patterns match
        matches = sum(1 for p in pattern_info['patterns'] if p in candid_summary)

        # If at least 2 patterns match, consider it identified
        if matches >= 2:
            return pattern_info['project'], pattern_info['category'], pattern_info['notes']

    return None, None, None

def set_project_backend(canister_id, project_name):
    """Set project name in backend."""
    try:
        cmd = [
            'dfx', 'canister', '--network', 'ic', 'call',
            BACKEND, 'set_project',
            f'(principal "{canister_id}", opt "{project_name}")'
        ]
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=60)
        return result.returncode == 0
    except Exception as e:
        print(f"  ⚠ Error setting project: {e}")
        return False

def process_unknowns():
    """Process all unknowns and try to identify them."""
    # Load unknowns
    unknowns = load_results(RESULTS_UNKNOWN)
    identified_results = load_results(RESULTS_IDENTIFIED)

    print(f"Processing {len(unknowns)} unknown canisters")

    newly_identified = 0
    still_unknown = []

    for entry in unknowns:
        canister_id = entry['canister_id']

        # Skip if already has a project (shouldn't happen but check anyway)
        if entry.get('project'):
            still_unknown.append(entry)
            continue

        # Try to identify based on Candid patterns
        candid_summary = entry.get('candid_summary', '')
        if candid_summary:
            project, category, notes = analyze_candid(candid_summary)

            if project:
                print(f"✓ {canister_id}: Identified as {project}")

                # Update entry
                entry['project'] = project
                entry['category'] = category
                entry['notes'] = notes
                entry['identified_by'] = 'candid_pattern_matching'
                entry['last_researched'] = datetime.now().strftime("%Y-%m-%d")
                del entry['reason']  # Remove unknown reason

                # Add to identified results
                identified_results.append(entry)
                newly_identified += 1

                # Set in backend
                set_project_backend(canister_id, project)
                continue

        # Still unknown
        still_unknown.append(entry)

    # Save results
    save_results(RESULTS_IDENTIFIED, identified_results)
    save_results(RESULTS_UNKNOWN, still_unknown)

    print(f"\n{'='*60}")
    print(f"Pattern Matching Complete")
    print(f"{'='*60}")
    print(f"  Newly identified: {newly_identified}")
    print(f"  Still unknown: {len(still_unknown)}")
    print(f"\n  Updated files:")
    print(f"    - {RESULTS_IDENTIFIED}")
    print(f"    - {RESULTS_UNKNOWN}")

if __name__ == '__main__':
    process_unknowns()
