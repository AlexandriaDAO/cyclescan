#!/bin/bash
# Backup all canister metadata from CycleScan
set -e

CANISTER_ID="cyclescan_backend"
OUTPUT_FILE="canister_metadata_backup.json"

echo "Exporting canister metadata..."
dfx canister --network ic call "$CANISTER_ID" export_canisters '()' --output json > "$OUTPUT_FILE"

COUNT=$(jq 'length' "$OUTPUT_FILE")
echo "✓ Successfully exported $COUNT canisters to $OUTPUT_FILE"
echo "  File size: $(du -h "$OUTPUT_FILE" | cut -f1)"
