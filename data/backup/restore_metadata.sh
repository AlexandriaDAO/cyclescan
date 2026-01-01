#!/bin/bash
# Restore canister metadata to CycleScan
set -e

CANISTER_ID="cyclescan_backend"
INPUT_FILE="${1:-canister_metadata_backup.json}"

if [ ! -f "$INPUT_FILE" ]; then
  echo "✗ Input file not found: $INPUT_FILE"
  echo "Usage: $0 [backup_file.json]"
  exit 1
fi

COUNT=$(jq 'length' "$INPUT_FILE")
echo "Importing $COUNT canisters from $INPUT_FILE..."

# Read the JSON array and pass it to import_canisters
IMPORT_DATA=$(cat "$INPUT_FILE")
dfx canister --network ic call "$CANISTER_ID" import_canisters "($IMPORT_DATA)"

echo "✓ Successfully imported $COUNT canisters"
