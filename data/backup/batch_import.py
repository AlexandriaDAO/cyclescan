#!/usr/bin/env python3
"""Import projects and canisters from backup files (new format only)."""
import json
import subprocess
import sys

BATCH_SIZE = 50
CANISTER_ID = "vohji-riaaa-aaaac-babxq-cai"

def run_dfx(method, candid):
    result = subprocess.run(
        ['dfx', 'canister', 'call', CANISTER_ID, method, candid, '--network', 'ic'],
        capture_output=True, text=True
    )
    if result.returncode != 0:
        print(f"Error: {result.stderr}")
        sys.exit(1)
    return result.stdout.strip()

def projects_to_candid(projects):
    items = []
    for p in projects:
        website = p.get('website')
        if isinstance(website, list):
            website = website[0] if website else None
        web = f'opt "{website}"' if website else 'null'
        items.append(f'record {{ name = "{p["name"]}"; website = {web} }}')
    return f'(vec {{ {"; ".join(items)} }})'

def canisters_to_candid(canisters):
    items = []
    for c in canisters:
        proxy_type = list(c['proxy_type'].keys())[0]
        project = c.get('project')
        if isinstance(project, list):
            project = project[0] if project else None
        proj = f'opt "{project}"' if project else 'null'
        valid = c.get('valid', True)
        if isinstance(valid, list):
            valid = valid[0] if valid else True
        items.append(
            f'record {{ canister_id = principal "{c["canister_id"]}"; '
            f'proxy_id = principal "{c["proxy_id"]}"; '
            f'proxy_type = variant {{ {proxy_type} }}; '
            f'project = {proj}; '
            f'valid = opt {"true" if valid else "false"} }}'
        )
    return f'(vec {{ {"; ".join(items)} }})'

with open('projects_backup.json') as f:
    projects = json.load(f)
with open('canisters_backup.json') as f:
    canisters = json.load(f)

print(f"Projects: {len(projects)}, Canisters: {len(canisters)}")

# Import projects
print(f"Importing {len(projects)} projects...")
print(f"  {run_dfx('import_projects', projects_to_candid(projects))}")

# Import canisters in batches
print(f"Importing {len(canisters)} canisters...")
for i in range(0, len(canisters), BATCH_SIZE):
    batch = canisters[i:i+BATCH_SIZE]
    print(f"  Batch {i}-{i+len(batch)}: {run_dfx('import_canisters', canisters_to_candid(batch))}")

print("\n✓ Done! Run: dfx canister call cyclescan_backend take_snapshot --network ic")
