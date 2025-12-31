#!/usr/bin/env python3
"""
Check coverage of research vs leaderboard unknowns.
"""

import json

# Load leaderboard unknowns
with open('/home/theseus/alexandria/cyclescan/data/unknowns_to_research.json', 'r') as f:
    leaderboard_unknowns = json.load(f)

leaderboard_ids = set(c['canister_id'] for c in leaderboard_unknowns)

# Load researched canisters
researched_ids = set()

# From research_results.json
try:
    with open('/home/theseus/alexandria/cyclescan/data/research_results.json', 'r') as f:
        data = json.load(f)
        researched_ids.update(c.get('canister_id') for c in data if c.get('canister_id'))
except Exception as e:
    print(f"Error loading research_results.json: {e}")

# From research_results2.json
try:
    with open('/home/theseus/alexandria/cyclescan/data/research_results2.json', 'r') as f:
        data = json.load(f)
        researched_ids.update(c.get('canister_id') for c in data if c.get('canister_id'))
except Exception as e:
    print(f"Error loading research_results2.json: {e}")

# From research_results_unknown.json
try:
    with open('/home/theseus/alexandria/cyclescan/data/research_results_unknown.json', 'r') as f:
        data = json.load(f)
        researched_ids.update(c.get('canister_id') for c in data if c.get('canister_id'))
except Exception as e:
    print(f"Error loading research_results_unknown.json: {e}")

print(f"Leaderboard unknowns: {len(leaderboard_ids)}")
print(f"Researched canisters: {len(researched_ids)}")

# Find overlaps
in_both = leaderboard_ids & researched_ids
in_leaderboard_not_researched = leaderboard_ids - researched_ids
in_researched_not_leaderboard = researched_ids - leaderboard_ids

print(f"\nOverlap (in both): {len(in_both)}")
print(f"In leaderboard but not researched: {len(in_leaderboard_not_researched)}")
print(f"In research but not in leaderboard: {len(in_researched_not_leaderboard)}")

if in_leaderboard_not_researched:
    print(f"\nFirst 10 in leaderboard but not researched:")
    for cid in list(in_leaderboard_not_researched)[:10]:
        print(f"  - {cid}")
