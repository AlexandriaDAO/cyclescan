#!/usr/bin/env python3
"""
Fetch logos from ICRC-1 token metadata and save to frontend static folder.
"""

import json
import subprocess
import base64
import os
import re
from pathlib import Path

# Output directory for logos
LOGO_DIR = Path("../src/cyclescan_frontend/static/logos")

def normalize_project_name(name):
    """Convert project name to safe filename."""
    return re.sub(r'[^a-z0-9]', '-', name.lower()).strip('-')

def get_one_canister_per_project():
    """Get one token canister for each unique project."""
    with open('research_results.json', 'r') as f:
        data = json.load(f)

    projects = {}
    for entry in data:
        if entry.get('is_token') and entry.get('project') and not entry['project'].startswith('Unknown'):
            project = entry['project']
            if project not in projects:
                projects[project] = entry['canister_id']

    return projects

def fetch_metadata(canister_id):
    """Query icrc1_metadata for a canister."""
    try:
        result = subprocess.run(
            ['dfx', 'canister', '--network', 'ic', 'call', canister_id, 'icrc1_metadata', '()'],
            capture_output=True,
            text=True,
            timeout=30
        )
        return result.stdout
    except Exception as e:
        print(f"  Error fetching metadata: {e}")
        return None

def extract_logo_from_metadata(metadata_str):
    """Extract logo data URI from metadata response."""
    # Look for icrc1:logo field with data URI
    match = re.search(r'"icrc1:logo"[^"]*"(data:image/[^"]+)"', metadata_str)
    if match:
        return match.group(1)
    return None

def save_logo(project, data_uri):
    """Save logo from data URI to file."""
    # Parse data URI: data:image/png;base64,... or data:image/svg+xml;base64,...
    match = re.match(r'data:image/([^;]+);base64,(.+)', data_uri)
    if not match:
        print(f"  Invalid data URI format")
        return False

    img_type = match.group(1)
    b64_data = match.group(2)

    # Determine extension
    if img_type == 'svg+xml':
        ext = 'svg'
    elif img_type == 'png':
        ext = 'png'
    elif img_type == 'jpeg' or img_type == 'jpg':
        ext = 'jpg'
    else:
        ext = img_type

    filename = f"{normalize_project_name(project)}.{ext}"
    filepath = LOGO_DIR / filename

    try:
        img_data = base64.b64decode(b64_data)
        with open(filepath, 'wb') as f:
            f.write(img_data)
        print(f"  Saved: {filename} ({len(img_data)} bytes)")
        return True
    except Exception as e:
        print(f"  Error saving logo: {e}")
        return False

def main():
    # Create output directory
    LOGO_DIR.mkdir(parents=True, exist_ok=True)

    projects = get_one_canister_per_project()
    print(f"Found {len(projects)} projects with tokens\n")

    success = 0
    failed = 0
    no_logo = 0

    for project, canister_id in sorted(projects.items()):
        print(f"{project} ({canister_id})...")

        metadata = fetch_metadata(canister_id)
        if not metadata:
            failed += 1
            continue

        logo_uri = extract_logo_from_metadata(metadata)
        if not logo_uri:
            print(f"  No logo in metadata")
            no_logo += 1
            continue

        if save_logo(project, logo_uri):
            success += 1
        else:
            failed += 1

    print(f"\nDone! Success: {success}, No logo: {no_logo}, Failed: {failed}")

if __name__ == '__main__':
    main()
