#!/bin/bash
# Import canisters in batches

set -e

export DFX_WARNING=-mainnet_plaintext_identity
cd "$(dirname "$0")"

for i in $(seq 0 15); do
    BATCH_FILE="batch_$i.json"
    if [ ! -f "$BATCH_FILE" ]; then
        echo "Skipping $BATCH_FILE (not found)"
        continue
    fi

    COUNT=$(jq 'length' "$BATCH_FILE")
    if [ "$COUNT" -eq 0 ]; then
        echo "Skipping $BATCH_FILE (empty)"
        continue
    fi

    echo "Importing batch $i ($COUNT canisters)..."

    # Convert JSON to Candid
    CANDID=$(jq -r '
    "(vec { " +
    (map("record { canister_id = principal \"\(.canister_id)\"; proxy_id = principal \"\(.proxy_id)\"; proxy_type = variant { \(.proxy_type) } }") | join("; ")) +
    " })"
    ' "$BATCH_FILE")

    # Import
    dfx canister call cyclescan_backend import_canisters "$CANDID" --network ic

    echo "Batch $i complete"
    sleep 1
done

echo "All batches imported!"
dfx canister call cyclescan_backend get_canister_count --network ic
