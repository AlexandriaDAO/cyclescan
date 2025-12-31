#!/usr/bin/env python3
"""
Consolidate all research results into master documentation and export formats.
Combines original 206 + new 2,433 canisters into unified database.
"""

import json
import csv
import re
from datetime import datetime

def load_original_research():
    """Load original 206 canisters from RESEARCH_PLAN.md"""
    canisters = {}

    with open('RESEARCH_PLAN.md', 'r') as f:
        for line in f:
            if '| done |' in line:
                parts = [p.strip() for p in line.split('|')]
                if len(parts) >= 6:
                    canister_id = parts[2]
                    project = parts[4]
                    notes = parts[5]

                    # Extract token info if present
                    token_match = re.search(r'Token: (.+?) \((.+?)\)', notes)
                    if token_match:
                        canisters[canister_id] = {
                            'project': project,
                            'type': 'token',
                            'token_name': token_match.group(1),
                            'token_symbol': token_match.group(2),
                            'notes': notes
                        }
                    else:
                        canisters[canister_id] = {
                            'project': project,
                            'type': 'backend' if project == 'Unknown' else 'other',
                            'token_name': None,
                            'token_symbol': None,
                            'notes': notes
                        }

    return canisters

def load_new_research():
    """Load new 2,433 canisters from research_results.json"""
    try:
        with open('research_results.json', 'r') as f:
            results = json.load(f)

        canisters = {}
        for item in results:
            cid = item['canister_id']
            canisters[cid] = {
                'project': item['project'],
                'type': 'token' if item.get('is_token') else 'backend',
                'token_name': item.get('token_name'),
                'token_symbol': item.get('token_symbol'),
                'notes': item['notes']
            }

        return canisters
    except FileNotFoundError:
        return {}

def create_project_mappings(all_canisters):
    """Create project_mappings.json with all data"""
    mappings = {
        'by_canister': {},
        'icrc1_tokens': {},
        'by_token_pattern': {
            'ODIN': 'ODIN.fun (Bioniq)',
            'WELL': 'FomoWell',
            '.OT': 'Ordi Trade'
        },
        'non_token_canisters': [],
        'unknown': []
    }

    for cid, data in all_canisters.items():
        # Add to by_canister
        if data['project'] not in ['Unknown', 'Unknown Token']:
            mappings['by_canister'][cid] = data['project']

        # Add to icrc1_tokens if it's a token
        if data['type'] == 'token' and data['token_name']:
            mappings['icrc1_tokens'][cid] = {
                'name': data['token_name'],
                'symbol': data['token_symbol'],
                'project': data['project'] if data['project'] not in ['Unknown Token'] else None
            }

        # Track non-tokens and unknowns
        if data['type'] != 'token':
            if data['project'] == 'Unknown':
                mappings['unknown'].append(cid)
            else:
                mappings['non_token_canisters'].append(cid)

    return mappings

def create_csv_export(all_canisters, filename='canister_projects.csv'):
    """Export all canisters to CSV"""
    with open(filename, 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow(['canister_id', 'project', 'type', 'token_name', 'token_symbol', 'notes'])

        for cid in sorted(all_canisters.keys()):
            data = all_canisters[cid]
            writer.writerow([
                cid,
                data['project'],
                data['type'],
                data['token_name'] or '',
                data['token_symbol'] or '',
                data['notes']
            ])

def update_master_doc(all_canisters):
    """Update MASTER_CANISTER_RESEARCH.md with final statistics"""
    # Calculate statistics
    total = len(all_canisters)
    tokens = sum(1 for d in all_canisters.values() if d['type'] == 'token')
    odin = sum(1 for d in all_canisters.values() if d['project'] == 'ODIN.fun')
    unknown_tokens = sum(1 for d in all_canisters.values() if d['project'] == 'Unknown Token')
    unknown = sum(1 for d in all_canisters.values() if d['project'] == 'Unknown')
    identified = total - unknown - unknown_tokens

    stats = f"""### Summary Statistics

- **Total Canisters:** {total:,}
- **Identified Projects:** {identified:,} ({100*identified/total:.1f}%)
- **ICRC-1 Tokens:** {tokens:,} ({100*tokens/total:.1f}%)
  - ODIN.fun: {odin:,}
  - Unknown Tokens: {unknown_tokens:,}
- **Unknown (Non-token):** {unknown:,} ({100*unknown/total:.1f}%)

**Last Updated:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}
"""

    # Read current doc
    with open('MASTER_CANISTER_RESEARCH.md', 'r') as f:
        content = f.read()

    # Replace summary section
    content = re.sub(
        r'### Summary Statistics.*?---',
        stats + '\n---',
        content,
        flags=re.DOTALL
    )

    # Write updated doc
    with open('MASTER_CANISTER_RESEARCH.md', 'w') as f:
        f.write(content)

    return stats

def main():
    print("=" * 60)
    print("CONSOLIDATING ALL RESEARCH RESULTS")
    print("=" * 60)
    print()

    print("Loading original research (206 canisters)...")
    original = load_original_research()
    print(f"  Loaded {len(original)} canisters")

    print("\nLoading new research (2,433 canisters)...")
    new = load_new_research()
    print(f"  Loaded {len(new)} canisters")

    print("\nCombining datasets...")
    all_canisters = {**original, **new}
    print(f"  Total: {len(all_canisters)} unique canisters")

    print("\nCreating project_mappings.json...")
    mappings = create_project_mappings(all_canisters)
    with open('project_mappings.json', 'w') as f:
        json.dump(mappings, f, indent=2)
    print(f"  ✓ Saved {len(mappings['icrc1_tokens'])} tokens")
    print(f"  ✓ Saved {len(mappings['by_canister'])} identified projects")

    print("\nCreating canister_projects.csv...")
    create_csv_export(all_canisters)
    print(f"  ✓ Exported {len(all_canisters)} rows")

    print("\nUpdating MASTER_CANISTER_RESEARCH.md...")
    stats = update_master_doc(all_canisters)
    print(stats)

    print("=" * 60)
    print("✅ CONSOLIDATION COMPLETE!")
    print("=" * 60)
    print("\nGenerated files:")
    print("  • project_mappings.json - JSON database")
    print("  • canister_projects.csv - CSV export")
    print("  • MASTER_CANISTER_RESEARCH.md - Master documentation")

if __name__ == '__main__':
    main()
