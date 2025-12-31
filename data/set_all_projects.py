#!/usr/bin/env python3
"""
Set all identified projects in the backend.
Processes research_results2.json and sets project names for all identified canisters.
"""

import json
import subprocess
import time

BACKEND = "vohji-riaaa-aaaac-babxq-cai"
RESULTS_IDENTIFIED = "/home/theseus/alexandria/cyclescan/data/research_results2.json"

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

def main():
    """Set all projects in backend."""
    # Load identified results
    with open(RESULTS_IDENTIFIED, 'r') as f:
        results = json.load(f)

    print(f"Setting {len(results)} projects in backend...")

    success = 0
    failed = 0

    for i, entry in enumerate(results, 1):
        canister_id = entry.get('canister_id')
        project = entry.get('project')

        if not canister_id or not project:
            print(f"  {i}. Skipping {canister_id} - no project name")
            continue

        print(f"  {i}/{len(results)}: Setting {canister_id} -> {project}")

        if set_project_backend(canister_id, project):
            success += 1
        else:
            failed += 1
            print(f"    ❌ Failed to set project")

        # Rate limiting
        if i % 10 == 0:
            time.sleep(1)

    print(f"\n{'='*60}")
    print(f"Complete!")
    print(f"{'='*60}")
    print(f"  Success: {success}")
    print(f"  Failed: {failed}")

if __name__ == '__main__':
    main()
