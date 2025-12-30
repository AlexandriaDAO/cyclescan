#!/usr/bin/env python3
"""
Automated canister research script.
Checks each canister for: ICRC-1 token, frontend, candid interface.
Updates RESEARCH_PLAN.md with findings.
"""

import subprocess
import json
import time
import re
from typing import Optional, Tuple
import requests

def check_icrc1_token(canister_id: str) -> Optional[Tuple[str, str]]:
    """Check if canister is an ICRC-1 token. Returns (name, symbol) if yes."""
    try:
        # Try to get token name
        result = subprocess.run(
            ['dfx', 'canister', '--network', 'ic', 'call', canister_id, 'icrc1_name', '()'],
            capture_output=True,
            text=True,
            timeout=10
        )
        if result.returncode == 0 and '(' in result.stdout:
            name = result.stdout.strip().strip('()').strip('"')

            # Try to get symbol
            result = subprocess.run(
                ['dfx', 'canister', '--network', 'ic', 'call', canister_id, 'icrc1_symbol', '()'],
                capture_output=True,
                text=True,
                timeout=10
            )
            if result.returncode == 0 and '(' in result.stdout:
                symbol = result.stdout.strip().strip('()').strip('"')
                return (name, symbol)
    except Exception as e:
        print(f"  ICRC-1 check failed: {e}")
    return None

def check_frontend(canister_id: str) -> Optional[str]:
    """Check if canister has a web frontend. Returns URL if accessible."""
    urls = [
        f"https://{canister_id}.icp0.io",
        f"https://{canister_id}.ic0.app",
        f"https://{canister_id}.raw.icp0.io"
    ]

    for url in urls:
        try:
            response = requests.head(url, timeout=5, allow_redirects=True)
            if response.status_code < 500:  # 200, 300, 400 codes might have content
                return url
        except:
            pass
    return None

def check_candid_interface(canister_id: str) -> Optional[str]:
    """Get candid interface methods. Returns method summary if available."""
    try:
        result = subprocess.run(
            ['dfx', 'canister', '--network', 'ic', 'call', canister_id, '__get_candid_interface_tmp_hack', '()'],
            capture_output=True,
            text=True,
            timeout=10
        )
        if result.returncode == 0:
            # Extract method names from candid interface
            methods = re.findall(r'(\w+)\s*:', result.stdout)
            if methods:
                return f"Methods: {', '.join(methods[:10])}"
    except:
        pass
    return None

def identify_project(canister_id: str, name: str = None, symbol: str = None) -> Tuple[str, str]:
    """Identify project name from token name/symbol or return Unknown."""
    if not name:
        return "Unknown", "No token interface, no frontend, no identifiable features"

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
    """Research a single canister and return findings."""
    print(f"\nResearching {canister_id}...")

    findings = {
        'canister_id': canister_id,
        'project': 'Unknown',
        'notes': ''
    }

    # Check if it's an ICRC-1 token
    print(f"  Checking ICRC-1...")
    token_info = check_icrc1_token(canister_id)

    if token_info:
        name, symbol = token_info
        print(f"  ✓ ICRC-1 Token: {name} ({symbol})")
        project, notes = identify_project(canister_id, name, symbol)
        findings['project'] = project
        findings['notes'] = notes
        return findings

    # Check for frontend
    print(f"  Checking frontend...")
    frontend_url = check_frontend(canister_id)
    if frontend_url:
        print(f"  ✓ Frontend found: {frontend_url}")
        findings['notes'] += f"Frontend: {frontend_url}. "

    # Check candid interface
    print(f"  Checking candid...")
    candid_info = check_candid_interface(canister_id)
    if candid_info:
        print(f"  ✓ {candid_info}")
        findings['notes'] += candid_info

    if not findings['notes']:
        findings['notes'] = "No token interface, no frontend, no candid interface"

    return findings

def load_canisters_from_plan() -> list:
    """Load canister IDs from RESEARCH_PLAN.md"""
    canisters = []
    with open('RESEARCH_PLAN.md', 'r') as f:
        for line in f:
            # Match table rows: | # | canister_id | pending | | |
            match = re.match(r'\|\s*\d+\s*\|\s*([a-z0-9-]+)\s*\|\s*pending\s*\|', line)
            if match:
                canisters.append(match.group(1))
    return canisters

def update_plan_table(canister_id: str, project: str, notes: str):
    """Update RESEARCH_PLAN.md with research results."""
    with open('RESEARCH_PLAN.md', 'r') as f:
        content = f.read()

    # Find and update the row for this canister
    pattern = r'(\|\s*\d+\s*\|\s*' + re.escape(canister_id) + r'\s*\|\s*)pending(\s*\|\s*\|\s*\|)'
    replacement = r'\1done\2' + f'{project} | {notes} |'

    # This won't work perfectly - need to update just the status column
    # Let's use a simpler approach: find the line and replace it
    lines = content.split('\n')
    for i, line in enumerate(lines):
        if canister_id in line and '| pending |' in line:
            parts = line.split('|')
            parts[3] = ' done '
            parts[4] = f' {project} '
            parts[5] = f' {notes} '
            lines[i] = '|'.join(parts)
            break

    with open('RESEARCH_PLAN.md', 'w') as f:
        f.write('\n'.join(lines))

def update_project_mappings(canister_id: str, project: str, name: str = None, symbol: str = None):
    """Update project_mappings.json with findings."""
    with open('project_mappings.json', 'r') as f:
        mappings = json.load(f)

    # Add to by_canister if not a token
    if not name:
        mappings['by_canister'][canister_id] = project
    else:
        # Add to icrc1_tokens
        mappings['icrc1_tokens'][canister_id] = {
            'name': name,
            'symbol': symbol,
            'project': project if project != "Unknown Token" else None
        }

    with open('project_mappings.json', 'w') as f:
        json.dump(mappings, f, indent=2)

def main():
    """Main research loop."""
    print("Starting canister research...")

    canisters = load_canisters_from_plan()
    print(f"Found {len(canisters)} canisters to research")

    for i, canister_id in enumerate(canisters, 1):
        print(f"\n[{i}/{len(canisters)}] Researching {canister_id}")

        findings = research_canister(canister_id)

        # Update tracking table
        update_plan_table(canister_id, findings['project'], findings['notes'])
        print(f"  Updated RESEARCH_PLAN.md")

        # Don't update project_mappings for every unknown - only for identified projects
        # We'll do a manual pass later

        # Rate limit to avoid overwhelming the network
        if i % 10 == 0:
            print(f"\n  Checkpoint: Completed {i}/{len(canisters)}")
            time.sleep(2)
        else:
            time.sleep(0.5)

    print(f"\n✓ Research complete! Processed {len(canisters)} canisters.")

if __name__ == '__main__':
    main()
