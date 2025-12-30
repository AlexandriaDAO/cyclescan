#!/bin/bash
# Fetch all SNS canisters and generate import data
# Output: sns_canisters.json - ready for import_canisters()

set -e

OUTPUT="sns_canisters.json"
SNS_WASM_CANISTER="qaa6y-5yaaa-aaaaa-aaafa-cai"

echo "Fetching list of deployed SNSes..."

# Get all SNS instances
dfx canister --network ic call $SNS_WASM_CANISTER list_deployed_snses '(record {})' 2>/dev/null > /tmp/sns_raw.txt

# Extract root canister IDs
grep -oE 'root_canister_id = opt principal "[a-z0-9-]+-cai"' /tmp/sns_raw.txt | \
  sed 's/root_canister_id = opt principal "//;s/"//' > /tmp/sns_roots.txt

SNS_COUNT=$(wc -l < /tmp/sns_roots.txt)
echo "Found $SNS_COUNT SNSes"

# For each SNS root, get all canister IDs
echo "["  > "$OUTPUT"
FIRST=true

while read -r SNS_ROOT; do
  echo "  Fetching $SNS_ROOT..."

  # Query the SNS root for its canisters summary
  SUMMARY=$(dfx canister --network ic call "$SNS_ROOT" get_sns_canisters_summary '(record {})' 2>/dev/null || echo "FAILED")

  if [[ "$SUMMARY" == "FAILED" ]]; then
    echo "    FAILED - skipping"
    continue
  fi

  # Extract canister IDs from the response
  # Core canisters: root, governance, ledger, index, swap
  for TYPE in root governance ledger index swap; do
    CANISTER=$(echo "$SUMMARY" | grep -oP "${TYPE} = opt record \{[^}]*canister_id = opt principal \"[a-z0-9-]+-cai\"" | grep -oP 'principal "[a-z0-9-]+-cai"' | head -1 | sed 's/principal "//;s/"//') || true
    if [[ -n "$CANISTER" ]]; then
      if [[ "$FIRST" == "true" ]]; then
        FIRST=false
      else
        echo "," >> "$OUTPUT"
      fi
      echo -n "  {\"canister_id\": \"$CANISTER\", \"proxy_id\": \"$SNS_ROOT\", \"proxy_type\": \"SnsRoot\"}" >> "$OUTPUT"
    fi
  done

  # Dapps (variable number)
  while read -r DAPP; do
    if [[ -n "$DAPP" ]]; then
      echo "," >> "$OUTPUT"
      echo -n "  {\"canister_id\": \"$DAPP\", \"proxy_id\": \"$SNS_ROOT\", \"proxy_type\": \"SnsRoot\"}" >> "$OUTPUT"
    fi
  done < <(echo "$SUMMARY" | grep -oP 'dapps = vec \{[^}]+\}' | grep -oP 'canister_id = opt principal "[a-z0-9-]+-cai"' | sed 's/canister_id = opt principal "//;s/"//g')

  # Archives (variable number)
  while read -r ARCHIVE; do
    if [[ -n "$ARCHIVE" ]]; then
      echo "," >> "$OUTPUT"
      echo -n "  {\"canister_id\": \"$ARCHIVE\", \"proxy_id\": \"$SNS_ROOT\", \"proxy_type\": \"SnsRoot\"}" >> "$OUTPUT"
    fi
  done < <(echo "$SUMMARY" | grep -oP 'archives = vec \{[^}]+\}' | grep -oP 'canister_id = opt principal "[a-z0-9-]+-cai"' | sed 's/canister_id = opt principal "//;s/"//g')

done < /tmp/sns_roots.txt

echo "" >> "$OUTPUT"
echo "]" >> "$OUTPUT"

# Validate JSON
if jq empty "$OUTPUT" 2>/dev/null; then
  CANISTER_COUNT=$(jq 'length' "$OUTPUT")
  echo ""
  echo "=== Results ==="
  echo "SNSes processed: $SNS_COUNT"
  echo "Canisters found: $CANISTER_COUNT"
  echo "Output: $OUTPUT"
else
  echo "ERROR: Generated invalid JSON"
  exit 1
fi
