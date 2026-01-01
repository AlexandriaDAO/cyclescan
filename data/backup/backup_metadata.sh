#!/bin/bash
# Backup all canister and project metadata from CycleScan
set -e

CANISTER_ID="vohji-riaaa-aaaac-babxq-cai"
CANISTERS_FILE="canisters_backup.json"
PROJECTS_FILE="projects_backup.json"

echo "Exporting project metadata..."
dfx canister --network ic call "$CANISTER_ID" export_projects '()' --output json > "$PROJECTS_FILE"
PROJ_COUNT=$(jq 'length' "$PROJECTS_FILE")
echo "✓ Exported $PROJ_COUNT projects to $PROJECTS_FILE"

echo "Exporting canister metadata..."
dfx canister --network ic call "$CANISTER_ID" export_canisters '()' --output json > "$CANISTERS_FILE"
CAN_COUNT=$(jq 'length' "$CANISTERS_FILE")
echo "✓ Exported $CAN_COUNT canisters to $CANISTERS_FILE"

echo ""
echo "Backup complete:"
echo "  Projects: $(du -h "$PROJECTS_FILE" | cut -f1)"
echo "  Canisters: $(du -h "$CANISTERS_FILE" | cut -f1)"
