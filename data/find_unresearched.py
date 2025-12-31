#!/usr/bin/env python3
"""Find which current unknowns haven't been researched yet."""

import json

# Load current unknowns
with open('/tmp/current_unknowns.json', 'r') as f:
    current_unknowns = set(json.load(f))

print(f"Current unknowns in leaderboard: {len(current_unknowns)}")

# Load researched canisters
researched = set()

for filename in ['research_results.json', 'research_results2.json', 'research_results_unknown.json']:
    try:
        with open(f'/home/theseus/alexandria/cyclescan/data/{filename}', 'r') as f:
            data = json.load(f)
            for entry in data:
                if 'canister_id' in entry:
                    researched.add(entry['canister_id'])
    except Exception as e:
        print(f"Error loading {filename}: {e}")

print(f"Total researched: {len(researched)}")

# Find unresearched
unresearched = current_unknowns - researched
researched_but_still_unknown = current_unknowns & researched

print(f"\nUnresearched (need to research): {len(unresearched)}")
print(f"Researched but still project=null: {len(researched_but_still_unknown)}")

# Save unresearched
unresearched_list = sorted(list(unresearched))
with open('/home/theseus/alexandria/cyclescan/data/truly_unresearched.json', 'w') as f:
    json.dump(unresearched_list, f, indent=2)

print(f"\nSaved {len(unresearched_list)} unresearched canisters to truly_unresearched.json")

if unresearched_list:
    print(f"\nFirst 20 unresearched:")
    for cid in unresearched_list[:20]:
        print(f"  - {cid}")

# Check researched but still unknown - these should have been set
if researched_but_still_unknown:
    print(f"\n⚠ {len(researched_but_still_unknown)} canisters were researched but still show project=null")
    print("First 10:")
    for cid in sorted(list(researched_but_still_unknown))[:10]:
        print(f"  - {cid}")
