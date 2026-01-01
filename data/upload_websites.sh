#!/bin/bash

# Upload all discovered websites to the backend

# First, get all canister IDs for each project
# Then call set_websites in batches

# This requires getting the export from backend and matching project names to canister IDs

echo "Fetching current canisters from backend..."
dfx canister --network ic call vohji-riaaa-aaaac-babxq-cai export_canisters > current_canisters_export.txt 2>&1

echo "Creating website update candid calls..."
python3 << 'PYTHON_SCRIPT'
import json
import re

# Load websites
with open('websites_for_import.json', 'r') as f:
    websites = json.load(f)

# Load canister export
with open('current_canisters_export.txt', 'r') as f:
    data = f.read()

# Extract canister_id and project from export
pattern = r'canister_id = principal "([^"]+)".*?project = opt "([^"]+)"'
matches = re.findall(pattern, data, re.DOTALL)

# Build mapping: project -> [canister_ids]
project_to_canisters = {}
for canister_id, project in matches:
    if project not in project_to_canisters:
        project_to_canisters[project] = []
    project_to_canisters[project].append(canister_id)

# Create website updates
updates = []
for project, website in websites.items():
    if project in project_to_canisters:
        for canister_id in project_to_canisters[project]:
            updates.append(f'(principal "{canister_id}", opt "{website}")')

# Split into batches of 100
batch_size = 100
batches = [updates[i:i + batch_size] for i in range(0, len(updates), batch_size)]

# Create shell script for each batch
for i, batch in enumerate(batches):
    candid_arg = f'(vec {{ {"; ".join(batch)} }})'
    with open(f'batch_{i+1}_websites.sh', 'w') as f:
        f.write(f'''#!/bin/bash
echo "Uploading batch {i+1}/{len(batches)} ({len(batch)} websites)..."
dfx canister --network ic call vohji-riaaa-aaaac-babxq-cai set_websites '{candid_arg}'
echo "Batch {i+1} complete!"
''')

print(f"Total updates: {len(updates)}")
print(f"Created {len(batches)} batch files")
print(f"Projects matched: {len([p for p in websites.keys() if p in project_to_canisters])}")
print(f"Projects not found: {[p for p in websites.keys() if p not in project_to_canisters]}")

PYTHON_SCRIPT

# Make batch files executable
chmod +x batch_*_websites.sh

echo ""
echo "===== Ready to upload ====="
echo "Run the batch files to upload websites:"
echo "  ./batch_1_websites.sh"
echo "  ./batch_2_websites.sh"
echo "  ... etc"
