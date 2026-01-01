#!/usr/bin/env python3

import subprocess
import re
from collections import defaultdict

print("Fetching all canisters from backend...")
result = subprocess.run(
    ["dfx", "canister", "--network", "ic", "call", "vohji-riaaa-aaaac-babxq-cai", "export_canisters"],
    capture_output=True,
    text=True,
    timeout=300
)

if result.returncode != 0:
    print(f"Error: {result.stderr}")
    exit(1)

data = result.stdout

# Extract all project names using regex
project_pattern = r'project = opt "([^"]+)"'
projects = re.findall(project_pattern, data)

# Count occurrences
project_counts = defaultdict(int)
for project in projects:
    project_counts[project] += 1

# Get unique projects
unique_projects = sorted(set(projects))

print(f"\nTotal canisters: {len(projects)}")
print(f"Unique projects: {len(unique_projects)}")
print(f"\n{'='*80}")
print("All unique projects:")
print(f"{'='*80}\n")

for i, project in enumerate(unique_projects, 1):
    count = project_counts[project]
    print(f"{i}. {project} ({count} canisters)")

# Save to file for reference
with open('unique_projects.txt', 'w') as f:
    for project in unique_projects:
        f.write(f"{project}\n")

print(f"\n{'-'*80}")
print(f"Saved {len(unique_projects)} unique projects to unique_projects.txt")
