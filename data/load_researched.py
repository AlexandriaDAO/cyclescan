#!/usr/bin/env python3
"""Load all already-researched canister IDs to avoid duplicate work."""

import json
import sys

def load_researched():
    """Load all canister IDs that have been researched."""
    researched = set()

    # Load research_results.json (identified)
    try:
        with open('/home/theseus/alexandria/cyclescan/data/research_results.json', 'r') as f:
            data = json.load(f)
            for entry in data:
                if 'canister_id' in entry:
                    researched.add(entry['canister_id'])
            print(f"Loaded {len(data)} from research_results.json")
    except Exception as e:
        print(f"Warning: Could not load research_results.json: {e}")

    # Load research_results2.json (identified)
    try:
        with open('/home/theseus/alexandria/cyclescan/data/research_results2.json', 'r') as f:
            data = json.load(f)
            for entry in data:
                if 'canister_id' in entry:
                    researched.add(entry['canister_id'])
            print(f"Loaded {len(data)} from research_results2.json")
    except Exception as e:
        print(f"Warning: Could not load research_results2.json: {e}")

    # Load research_results_unknown.json (researched but unknown)
    try:
        with open('/home/theseus/alexandria/cyclescan/data/research_results_unknown.json', 'r') as f:
            data = json.load(f)
            for entry in data:
                if 'canister_id' in entry:
                    researched.add(entry['canister_id'])
            print(f"Loaded {len(data)} from research_results_unknown.json")
    except Exception as e:
        print(f"Warning: Could not load research_results_unknown.json: {e}")

    return researched

if __name__ == '__main__':
    # Load already researched
    researched = load_researched()
    print(f"\nTotal unique researched canisters: {len(researched)}")

    # Load unknowns to research
    with open('/home/theseus/alexandria/cyclescan/data/unknowns_to_research.json', 'r') as f:
        unknowns = json.load(f)

    print(f"Total unknowns from leaderboard: {len(unknowns)}")

    # Filter out already researched
    needs_research = [u for u in unknowns if u['canister_id'] not in researched]
    print(f"Needs research (new): {len(needs_research)}")

    # Save filtered list
    with open('/home/theseus/alexandria/cyclescan/data/needs_research.json', 'w') as f:
        json.dump(needs_research, f, indent=2)

    print(f"\nSaved {len(needs_research)} canisters to needs_research.json")

    if len(needs_research) > 0:
        print(f"\nTop 10 to research:")
        for i, canister in enumerate(needs_research[:10], 1):
            print(f"{i}. {canister['canister_id']}: {canister['burn_24h']:,} cycles/24h")
