#!/bin/bash

# Extract all canisters and find unique projects
dfx canister --network ic call vohji-riaaa-aaaac-babxq-cai export_canisters | \
  grep 'project = opt' | \
  sed 's/.*project = opt "\(.*\)".*/\1/' | \
  sort -u > unique_projects.txt

echo "Total unique projects:"
wc -l unique_projects.txt
echo ""
echo "First 50 projects:"
head -50 unique_projects.txt
