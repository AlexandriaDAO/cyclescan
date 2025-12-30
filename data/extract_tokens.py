#!/usr/bin/env python3
"""Extract token data from RESEARCH_PLAN.md and update project_mappings.json"""

import json
import re

def extract_tokens_from_plan():
    """Parse RESEARCH_PLAN.md and extract token information."""
    tokens = []

    with open('RESEARCH_PLAN.md', 'r') as f:
        for line in f:
            if '| done |' in line and 'Token:' in line:
                parts = [p.strip() for p in line.split('|')]
                if len(parts) >= 6:
                    canister_id = parts[2]
                    project = parts[4]  # parts[3] is status 'done'
                    notes = parts[5]    # notes are in column 5

                    # Extract name and symbol from "Token: Name (SYMBOL)"
                    token_match = re.search(r'Token: (.+?) \((.+?)\)', notes)
                    if token_match:
                        name = token_match.group(1)
                        symbol = token_match.group(2)

                        tokens.append({
                            'canister_id': canister_id,
                            'name': name,
                            'symbol': symbol,
                            'project': project if project not in ['Unknown', 'Unknown Token'] else None
                        })

    return tokens

def update_project_mappings(tokens):
    """Update project_mappings.json with token data."""
    with open('project_mappings.json', 'r') as f:
        mappings = json.load(f)

    # Add all tokens to icrc1_tokens section
    for token in tokens:
        mappings['icrc1_tokens'][token['canister_id']] = {
            'name': token['name'],
            'symbol': token['symbol'],
            'project': token['project']
        }

    # Count projects
    project_counts = {}
    for token in tokens:
        proj = token['project'] or 'Unknown'
        project_counts[proj] = project_counts.get(proj, 0) + 1

    with open('project_mappings.json', 'w') as f:
        json.dump(mappings, f, indent=2)

    return project_counts

def main():
    print("Extracting tokens from RESEARCH_PLAN.md...")
    tokens = extract_tokens_from_plan()
    print(f"Found {len(tokens)} tokens")

    print("\nUpdating project_mappings.json...")
    project_counts = update_project_mappings(tokens)

    print("\nProject distribution:")
    for project, count in sorted(project_counts.items(), key=lambda x: -x[1]):
        print(f"  {project}: {count}")

    print("\n✓ Updated project_mappings.json")

if __name__ == '__main__':
    main()
