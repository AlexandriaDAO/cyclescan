#!/usr/bin/env python3
"""
Research all 171 'Unidentified (needs review)' canisters.
Follows RESEARCH_UNKNOWNS.md methodology.
"""

import subprocess
import json
import time
import re
import sys
from typing import Optional, Tuple, Dict

BACKEND = "vohji-riaaa-aaaac-babxq-cai"

def run_dfx(args: list, timeout: int = 15) -> Tuple[bool, str, str]:
    """Run dfx command and return (success, stdout, stderr)."""
    try:
        result = subprocess.run(
            ['dfx', 'canister', '--network', 'ic'] + args,
            capture_output=True,
            text=True,
            timeout=timeout
        )
        return result.returncode == 0, result.stdout, result.stderr
    except subprocess.TimeoutExpired:
        return False, "", "Timeout"
    except Exception as e:
        return False, "", str(e)

def parse_dfx_string(stdout: str) -> Optional[str]:
    """Parse dfx output for a string result, handling WARN lines."""
    lines = [line.strip() for line in stdout.strip().split('\n') if line.strip()]
    if not lines:
        return None
    result_line = lines[-1]
    if result_line.startswith('("') and result_line.endswith('")'):
        return result_line[2:-2]
    return None

def parse_dfx_principal(stdout: str) -> Optional[str]:
    """Parse dfx output for a principal result."""
    match = re.search(r'principal "([^"]+)"', stdout)
    if match:
        return match.group(1)
    return None

def get_candid(canister_id: str) -> Optional[str]:
    """Get candid:service metadata."""
    success, stdout, stderr = run_dfx(['metadata', canister_id, 'candid:service'])
    if success and stdout.strip():
        return stdout
    return None

def check_icrc1_token(canister_id: str) -> Optional[Tuple[str, str]]:
    """Check if canister is ICRC-1 token. Returns (name, symbol) or None."""
    success, stdout, stderr = run_dfx(['call', canister_id, 'icrc1_name', '()'])
    if not success:
        return None
    name = parse_dfx_string(stdout)
    if not name:
        return None

    success, stdout, stderr = run_dfx(['call', canister_id, 'icrc1_symbol', '()'])
    if success:
        symbol = parse_dfx_string(stdout)
        if symbol:
            return (name, symbol)
    return None

def check_ledger_id(canister_id: str) -> Optional[str]:
    """Check if this is an index canister by calling ledger_id()."""
    success, stdout, stderr = run_dfx(['call', canister_id, 'ledger_id', '()'])
    if success:
        ledger = parse_dfx_principal(stdout)
        return ledger
    return None

def check_get_canisters(canister_id: str) -> Optional[str]:
    """Check if this is an SNS root or similar by calling get_canisters()."""
    success, stdout, stderr = run_dfx(['call', canister_id, 'get_canisters', '()'], timeout=20)
    if success and 'principal' in stdout:
        return "SNS-like"
    return None

def identify_from_candid(candid: str) -> Optional[str]:
    """Try to identify project from candid interface patterns."""
    candid_lower = candid.lower()

    # Check for common patterns
    if 'station' in candid_lower and 'request' in candid_lower:
        return "Orbit Station"
    if 'get_sns_' in candid_lower or 'sns_' in candid_lower:
        return "SNS Component"
    if 'swap' in candid_lower and 'pool' in candid_lower:
        return "DEX"
    if 'chain' in candid_lower and 'directive' in candid_lower:
        return "OmniBTC"
    if 'nft' in candid_lower and ('collection' in candid_lower or 'token_metadata' in candid_lower):
        return "NFT"
    if 'logger' in candid_lower or 'log_' in candid_lower:
        return "Logger Canister"
    if 'http_request' in candid and 'certified_tree' in candid_lower:
        return "Asset Canister"

    return None

def identify_token_project(name: str, symbol: str) -> str:
    """Identify project from token name/symbol."""
    name_upper = name.upper()
    symbol_upper = symbol.upper()

    # Known patterns
    if 'ODIN' in name_upper or '•ODIN' in symbol_upper:
        return "ODIN.fun"
    if 'WELL' in symbol_upper or 'FOMO' in name_upper:
        return "FomoWell"
    if '.OT' in symbol_upper:
        return "Ordi Trade"
    if 'SNS' in symbol_upper or 'NEURON' in name_upper:
        return "SNS Token"

    # Return symbol as token name
    return f"{symbol} Token"

def research_canister(canister_id: str) -> Dict:
    """Research a single canister and return findings."""
    result = {
        'canister_id': canister_id,
        'project': None,
        'category': None,
        'notes': []
    }

    # Step 1: Check if ICRC-1 token
    token_info = check_icrc1_token(canister_id)
    if token_info:
        name, symbol = token_info
        result['notes'].append(f"ICRC-1 Token: {name} ({symbol})")
        result['project'] = identify_token_project(name, symbol)
        return result

    # Step 2: Check if index canister
    ledger = check_ledger_id(canister_id)
    if ledger:
        result['notes'].append(f"Index for ledger: {ledger}")
        # Try to identify the ledger
        token_info = check_icrc1_token(ledger)
        if token_info:
            name, symbol = token_info
            result['project'] = f"{symbol} Index"
            result['notes'].append(f"Ledger is: {name} ({symbol})")
        else:
            result['project'] = "Index Canister"
        return result

    # Step 3: Get candid interface
    candid = get_candid(canister_id)
    if candid:
        result['notes'].append("Has candid interface")

        # Try to identify from candid
        project = identify_from_candid(candid)
        if project:
            result['project'] = project
            return result

        # Extract method names for notes
        methods = re.findall(r'(\w+)\s*:', candid)
        if methods:
            unique_methods = list(set(methods))[:10]
            result['notes'].append(f"Methods: {', '.join(unique_methods)}")
    else:
        result['notes'].append("No candid:service metadata")
        result['category'] = "Unidentified (no candid)"

    # Step 4: If still unidentified, mark appropriately
    if not result['project'] and not result['category']:
        result['category'] = "Unidentified"

    return result

def set_project(canister_id: str, project: str) -> bool:
    """Set project name on backend."""
    success, stdout, stderr = run_dfx([
        'call', BACKEND, 'set_project',
        f'(principal "{canister_id}", opt "{project}")'
    ])
    return success

def main():
    # Load canisters to research
    with open('/tmp/needs_review_canisters.txt', 'r') as f:
        canisters = [line.strip() for line in f if line.strip()]

    print(f"Researching {len(canisters)} canisters...")

    results = []
    updated = 0

    for i, canister_id in enumerate(canisters, 1):
        print(f"\n[{i}/{len(canisters)}] {canister_id}")

        finding = research_canister(canister_id)
        results.append(finding)

        # Determine final project name
        final_project = finding.get('project') or finding.get('category') or "Unidentified"

        print(f"  -> {final_project}")
        if finding['notes']:
            for note in finding['notes'][:2]:
                print(f"     {note}")

        # Update backend
        if set_project(canister_id, final_project):
            updated += 1
        else:
            print(f"  ! Failed to update backend")

        # Rate limit
        if i % 20 == 0:
            print(f"\nCheckpoint: {i}/{len(canisters)} processed, {updated} updated")
            time.sleep(1)
        else:
            time.sleep(0.3)

    # Save results
    with open('/tmp/research_results_batch.json', 'w') as f:
        json.dump(results, f, indent=2)

    print(f"\n{'='*50}")
    print(f"Complete! Processed {len(canisters)} canisters, updated {updated}")

    # Summary
    projects = {}
    for r in results:
        p = r.get('project') or r.get('category') or 'Unidentified'
        projects[p] = projects.get(p, 0) + 1

    print("\nProject distribution:")
    for p, count in sorted(projects.items(), key=lambda x: -x[1]):
        print(f"  {p}: {count}")

if __name__ == '__main__':
    main()
