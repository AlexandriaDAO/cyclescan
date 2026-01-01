#!/usr/bin/env python3
import json
import subprocess
import sys

BATCH_SIZE = 50

def to_candid(canisters):
    """Convert JSON to Candid format"""
    items = []
    for c in canisters:
        proxy_type = list(c['proxy_type'].keys())[0]
        project = f'opt "{c["project"][0]}"' if c.get('project') else 'null'
        website = f'opt "{c["website"][0]}"' if c.get('website') else 'null'
        items.append(
            f'record {{ canister_id = principal "{c["canister_id"]}"; '
            f'proxy_id = principal "{c["proxy_id"]}"; '
            f'proxy_type = variant {{ {proxy_type} }}; '
            f'project = {project}; '
            f'website = {website} }}'
        )
    return f'(vec {{ {"; ".join(items)} }})'

with open('canister_metadata_backup.json') as f:
    canisters = json.load(f)

total = len(canisters)
print(f"Total canisters: {total}")

for i in range(0, total, BATCH_SIZE):
    batch = canisters[i:i+BATCH_SIZE]
    print(f"Importing batch {i}-{i+len(batch)} ({len(batch)} canisters)...")

    candid = to_candid(batch)
    result = subprocess.run(
        ['dfx', 'canister', 'call', 'cyclescan_backend', 'import_canisters', candid, '--network', 'ic'],
        capture_output=True, text=True
    )
    if result.returncode != 0:
        print(f"Error: {result.stderr}")
        sys.exit(1)
    print(f"  {result.stdout.strip()}")

print("Done!")
