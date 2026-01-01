#!/bin/bash

# Backup all canister metadata from CycleScan
# This exports canister_id, proxy_id, proxy_type, project, and website
# The output can be re-imported using import_canisters()

CANISTER_ID="vohji-riaaa-aaaac-babxq-cai"
OUTPUT_FILE="canister_metadata_backup.json"

echo "Exporting canister metadata from $CANISTER_ID..."

dfx canister --network ic call "$CANISTER_ID" export_canisters '()' \
  --output json > "$OUTPUT_FILE"

if [ $? -eq 0 ]; then
  COUNT=$(jq 'length' "$OUTPUT_FILE")
  echo "✓ Successfully exported $COUNT canisters to $OUTPUT_FILE"
  echo "  File size: $(du -h "$OUTPUT_FILE" | cut -f1)"
else
  echo "✗ Export failed"
  exit 1
fi
