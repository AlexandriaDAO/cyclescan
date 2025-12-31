#!/bin/bash
# Set project names from research_results.json

cd "$(dirname "$0")"

# Extract canisters with identified projects (not Unknown)
jq -r '[.[] | select(.project | test("^Unknown") | not) | {canister_id, project}]' research_results.json > /tmp/projects_to_set.json

TOTAL=$(jq 'length' /tmp/projects_to_set.json)
echo "Setting projects for $TOTAL canisters..."

BATCH_SIZE=100
OFFSET=0

while [ $OFFSET -lt $TOTAL ]; do
    # Extract batch and format for Candid
    BATCH=$(jq -r --argjson offset $OFFSET --argjson size $BATCH_SIZE \
        '.[$offset:$offset+$size] | map("record { principal \"\(.canister_id)\"; opt \"\(.project)\" }") | join("; ")' \
        /tmp/projects_to_set.json)
    
    BATCH_COUNT=$(jq --argjson offset $OFFSET --argjson size $BATCH_SIZE '.[$offset:$offset+$size] | length' /tmp/projects_to_set.json)
    
    echo "Setting batch $((OFFSET/BATCH_SIZE + 1)): $BATCH_COUNT canisters (offset $OFFSET)..."
    
    RESULT=$(dfx canister --network ic call vohji-riaaa-aaaac-babxq-cai set_projects "(vec { $BATCH })" 2>&1)
    echo "  Result: $RESULT"
    
    OFFSET=$((OFFSET + BATCH_SIZE))
done

echo "Done!"
