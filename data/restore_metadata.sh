#!/bin/bash

# Restore canister metadata to CycleScan
# Imports from the backup JSON file created by backup_metadata.sh

CANISTER_ID="vohji-riaaa-aaaac-babxq-cai"
INPUT_FILE="${1:-canister_metadata_backup.json}"

if [ ! -f "$INPUT_FILE" ]; then
  echo "✗ Input file not found: $INPUT_FILE"
  echo "Usage: $0 [backup_file.json]"
  exit 1
fi

COUNT=$(jq 'length' "$INPUT_FILE")
echo "Importing $COUNT canisters from $INPUT_FILE..."

# Read the JSON array and pass it directly to import_canisters
IMPORT_DATA=$(cat "$INPUT_FILE")

dfx canister --network ic call "$CANISTER_ID" import_canisters "(${IMPORT_DATA})"

if [ $? -eq 0 ]; then
  echo "✓ Successfully imported canisters"
else
  echo "✗ Import failed"
  exit 1
fi
