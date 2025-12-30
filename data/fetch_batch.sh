#!/bin/bash
# Fetch a batch of canisters
# Usage: ./fetch_batch.sh START END

START=${1:?Usage: $0 START END}
END=${2:?Usage: $0 START END}
OUTPUT="canisters_${START}_${END}.json"
TEMP="canisters_${START}_${END}.jsonl"

echo "Fetching canisters $START to $END..."

for offset in $(seq $START 100 $((END - 100))); do
  curl -s "https://ic-api.internetcomputer.org/api/v3/canisters?limit=100&offset=$offset" | jq -c '.data[]' >> "$TEMP" 2>/dev/null

  if [ $((offset % 5000)) -eq 0 ]; then
    echo "Fetched offset $offset..."
  fi

  sleep 0.05
done

echo "Converting to JSON..."
jq -s '.' "$TEMP" > "$OUTPUT"
rm "$TEMP"

echo "Done!"
jq 'length' "$OUTPUT"
ls -lh "$OUTPUT"
