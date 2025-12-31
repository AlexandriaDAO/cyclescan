#!/usr/bin/env python3
"""
Research script for 2,433 new trackable canisters.
Reuses methodology from previous research but processes new batch.
"""

import subprocess
import json
import time
import re
from typing import Optional, Tuple
import requests

def parse_dfx_result(stdout: str) -> Optional[str]:
    """Parse dfx output, handling WARN lines that go to stdout.

    dfx outputs warnings to stdout (not stderr!) when candid:service
    metadata is missing. Example:
        WARN: Cannot fetch Candid interface for icrc1_name...
        ("Token")

    We extract only the actual result, which is the last non-empty line.
    """
    lines = [line.strip() for line in stdout.strip().split('\n') if line.strip()]
    if not lines:
        return None
    # Get the last line (the actual result, not warnings)
    result_line = lines[-1]
    # Ensure it looks like a valid result: ("...")
    if result_line.startswith('("') and result_line.endswith('")'):
        return result_line[2:-2]  # Extract content between ("...")
    elif result_line.startswith('(') and result_line.endswith(')'):
        # Handle simple values like (123)
        return result_line[1:-1].strip('"')
    return None


def check_icrc1_token(canister_id: str) -> Optional[Tuple[str, str]]:
    """Check if canister is an ICRC-1 token. Returns (name, symbol) if yes."""
    try:
        result = subprocess.run(
            ['dfx', 'canister', '--network', 'ic', 'call', canister_id, 'icrc1_name', '()'],
            capture_output=True,
            text=True,
            timeout=10
        )
        if result.returncode == 0:
            name = parse_dfx_result(result.stdout)
            if name is None:
                return None

            result = subprocess.run(
                ['dfx', 'canister', '--network', 'ic', 'call', canister_id, 'icrc1_symbol', '()'],
                capture_output=True,
                text=True,
                timeout=10
            )
            if result.returncode == 0:
                symbol = parse_dfx_result(result.stdout)
                if symbol:
                    return (name, symbol)
    except Exception as e:
        pass
    return None

def check_frontend(canister_id: str) -> Optional[str]:
    """Check if canister has a web frontend."""
    urls = [
        f"https://{canister_id}.icp0.io",
        f"https://{canister_id}.ic0.app",
        f"https://{canister_id}.raw.icp0.io"
    ]

    for url in urls:
        try:
            response = requests.head(url, timeout=5, allow_redirects=True)
            if response.status_code < 500:
                return url
        except:
            pass
    return None

def identify_project(canister_id: str, name: str = None, symbol: str = None) -> Tuple[str, str]:
    """Identify project name from token name/symbol."""
    if not name:
        return "Unknown", "No token interface"

    # Check patterns
    if "ODIN" in name or "ODIN" in symbol:
        return "ODIN.fun", f"Token: {name} ({symbol})"
    elif "WELL" in symbol:
        return "FomoWell", f"Token: {name} ({symbol})"
    elif ".OT" in symbol:
        return "Ordi Trade", f"Token: {name} ({symbol})"
    else:
        return "Unknown Token", f"Token: {name} ({symbol})"

def research_canister(canister_id: str) -> dict:
    """Research a single canister."""
    findings = {
        'canister_id': canister_id,
        'project': 'Unknown',
        'notes': '',
        'is_token': False,
        'token_name': None,
        'token_symbol': None
    }

    # Check ICRC-1
    token_info = check_icrc1_token(canister_id)
    if token_info:
        name, symbol = token_info
        project, notes = identify_project(canister_id, name, symbol)
        findings['project'] = project
        findings['notes'] = notes
        findings['is_token'] = True
        findings['token_name'] = name
        findings['token_symbol'] = symbol
        return findings

    # Check frontend
    frontend_url = check_frontend(canister_id)
    if frontend_url:
        findings['notes'] = f"Frontend: {frontend_url}"
    else:
        findings['notes'] = "No accessible frontend"

    return findings

def load_new_canisters() -> list:
    """Load list of new canisters to research."""
    with open('/tmp/new_canisters.txt', 'r') as f:
        return [line.strip() for line in f if line.strip()]

def save_progress(results: list, filename='research_results.json'):
    """Save research results to JSON."""
    with open(filename, 'w') as f:
        json.dump(results, f, indent=2)

def main():
    print("=" * 60)
    print("RESEARCHING 2,433 NEW TRACKABLE CANISTERS")
    print("=" * 60)
    print()

    canisters = load_new_canisters()
    print(f"Loaded {len(canisters)} new canisters to research\n")

    results = []
    tokens_found = 0
    unknowns = 0

    for i, canister_id in enumerate(canisters, 1):
        if i % 10 == 0:
            print(f"[{i}/{len(canisters)}] Progress checkpoint")
            save_progress(results)

        if i % 100 == 0:
            print(f"\n  Stats so far:")
            print(f"    Tokens found: {tokens_found}")
            print(f"    Unknown: {unknowns}")
            print(f"    Completion: {i}/{len(canisters)} ({100*i/len(canisters):.1f}%)\n")

        try:
            findings = research_canister(canister_id)
            results.append(findings)

            if findings['is_token']:
                tokens_found += 1
                if tokens_found <= 5:  # Show first 5 tokens
                    print(f"  ✓ Token: {findings['token_name']} ({findings['project']})")
            else:
                unknowns += 1

            # Rate limiting
            if i % 10 == 0:
                time.sleep(2)
            else:
                time.sleep(0.5)

        except Exception as e:
            print(f"  ✗ Error researching {canister_id}: {e}")
            results.append({
                'canister_id': canister_id,
                'project': 'Error',
                'notes': str(e),
                'is_token': False
            })

    # Save final results
    save_progress(results)

    print("\n" + "=" * 60)
    print("RESEARCH COMPLETE!")
    print("=" * 60)
    print(f"\nTotal researched: {len(results)}")
    print(f"Tokens found: {tokens_found}")
    print(f"Unknown: {unknowns}")
    print(f"\nResults saved to: research_results.json")

if __name__ == '__main__':
    main()
