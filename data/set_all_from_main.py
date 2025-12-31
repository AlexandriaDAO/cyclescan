#!/usr/bin/env python3
"""
Set all projects from research_results.json in the backend.
"""

import json
import subprocess
import time

BACKEND = "vohji-riaaa-aaaac-babxq-cai"

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
        return False

def main():
    """Set all projects in backend."""
    # Load research results
    with open('/home/theseus/alexandria/cyclescan/data/research_results.json', 'r') as f:
        results = json.load(f)

    print(f"Processing {len(results)} canisters from research_results.json...")

    success = 0
    failed = 0
    skipped = 0

    for i, entry in enumerate(results, 1):
        canister_id = entry.get('canister_id')
        project = entry.get('project')

        if not canister_id:
            skipped += 1
            continue

        if not project or project == "Unknown":
            skipped += 1
            continue

        if i % 100 == 0:
            print(f"  Progress: {i}/{len(results)} ({success} success, {failed} failed, {skipped} skipped)")

        if set_project_backend(canister_id, project):
            success += 1
        else:
            failed += 1

        # Rate limiting
        if i % 10 == 0:
            time.sleep(1)

    print(f"\n{'='*60}")
    print(f"Complete!")
    print(f"{'='*60}")
    print(f"  Success: {success}")
    print(f"  Failed: {failed}")
    print(f"  Skipped: {skipped}")

if __name__ == '__main__':
    main()
