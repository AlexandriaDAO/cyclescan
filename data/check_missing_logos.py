#!/usr/bin/env python3
"""
Check which projects are missing logos and suggest sources
"""

import json
import re
from pathlib import Path

DATA_DIR = Path(__file__).parent
LOGO_DIR = DATA_DIR.parent / "src/cyclescan_frontend/static/logos"
BACKUP_FILE = DATA_DIR / "backup/projects_backup.json"

def normalize_project_name(name: str) -> str:
    """Convert project name to kebab-case filename"""
    return re.sub(r'^-|-$', '', re.sub(r'-+', '-', re.sub(r'[^a-z0-9]', '-', name.lower())))

def main():
    with open(BACKUP_FILE) as f:
        projects = json.load(f)

    print("CycleScan Missing Logos Report")
    print("=" * 70)
    print()

    existing_logos = {f.stem for f in LOGO_DIR.glob("*.png")}

    has_logo = []
    missing_with_website = []
    missing_without_website = []

    for project in projects:
        name = project["name"]
        website = project["website"][0] if project["website"] else None
        filename = normalize_project_name(name)

        if filename in existing_logos:
            has_logo.append(name)
        elif website:
            missing_with_website.append((name, website))
        else:
            missing_without_website.append(name)

    print(f"✓ Projects WITH logos: {len(has_logo)}/{len(projects)}")
    print(f"✗ Missing logos (has website): {len(missing_with_website)}")
    print(f"✗ Missing logos (no website): {len(missing_without_website)}")
    print()

    if missing_with_website:
        print("MISSING LOGOS - Projects with Websites")
        print("-" * 70)
        for name, website in missing_with_website:
            print(f"  {name}")
            print(f"    Website: {website}")
            print(f"    Filename: {normalize_project_name(name)}.png")
            print()

    if missing_without_website:
        print("MISSING LOGOS - Projects WITHOUT Websites")
        print("-" * 70)
        for name in missing_without_website:
            print(f"  {name} → {normalize_project_name(name)}.png")
        print()

    # Export CSV for easy manual work
    csv_path = DATA_DIR / "missing_logos.csv"
    with open(csv_path, "w") as f:
        f.write("project_name,website,filename\n")
        for name, website in missing_with_website:
            f.write(f'"{name}","{website}","{normalize_project_name(name)}.png"\n')
        for name in missing_without_website:
            f.write(f'"{name}","","{normalize_project_name(name)}.png"\n')

    print(f"Exported to: {csv_path}")

if __name__ == "__main__":
    main()
