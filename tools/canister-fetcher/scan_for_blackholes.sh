#!/bin/bash
# Scan canisters for blackhole controllers using IC Dashboard API
# This is much faster than calling dfx for each canister

set -e

# Configuration
OUTPUT_DIR="data/02_trackable"
BATCH_SIZE=100
START_OFFSET=${1:-0}
END_OFFSET=${2:-25000}
SLEEP_BETWEEN=0.1  # Rate limiting

# Blackhole controller IDs
BLACKHOLES=(
  "e3mmv-5qaaa-aaaah-aadma-cai"  # ninegua Original
  "5vdms-kaaaa-aaaap-aa3uq-cai"  # CycleOps V1
  "2daxo-giaaa-aaaap-anvca-cai"  # CycleOps V2
  "cpbhu-5iaaa-aaaad-aalta-cai"  # CycleOps V3
  "w7sux-siaaa-aaaai-qpasa-cai"  # Cygnus
  "r7inp-6aaaa-aaaaa-aaabq-cai"  # NNS Root
)

# Build grep pattern
PATTERN=$(IFS='|'; echo "${BLACKHOLES[*]}")

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Output files
TRACKABLE_FILE="$OUTPUT_DIR/trackable_${START_OFFSET}_${END_OFFSET}.jsonl"
PROGRESS_FILE="$OUTPUT_DIR/scan_progress_${START_OFFSET}_${END_OFFSET}.json"

# Initialize or resume
if [ -f "$PROGRESS_FILE" ]; then
  CURRENT_OFFSET=$(jq -r '.current_offset' "$PROGRESS_FILE")
  echo "Resuming from offset $CURRENT_OFFSET"
else
  CURRENT_OFFSET=$START_OFFSET
  echo '{"current_offset": '$START_OFFSET', "total_scanned": 0, "total_trackable": 0}' > "$PROGRESS_FILE"
fi

total_scanned=0
total_trackable=0

echo "Scanning canisters from offset $CURRENT_OFFSET to $END_OFFSET..."
echo "Looking for blackhole controllers: ${BLACKHOLES[*]}"
echo ""

while [ $CURRENT_OFFSET -lt $END_OFFSET ]; do
  # Fetch batch
  response=$(curl -s "https://ic-api.internetcomputer.org/api/v3/canisters?limit=$BATCH_SIZE&offset=$CURRENT_OFFSET")

  # Check for errors
  if ! echo "$response" | jq -e '.data' > /dev/null 2>&1; then
    echo "Error fetching offset $CURRENT_OFFSET, retrying in 5s..."
    sleep 5
    continue
  fi

  batch_count=$(echo "$response" | jq '.data | length')

  # Filter for blackhole controllers and append to output (use any() to avoid duplicates when canister has multiple blackhole controllers)
  matches=$(echo "$response" | jq -c ".data[] | select(any(.controllers[]?; test(\"$PATTERN\")))")
  match_count=$(echo "$matches" | grep -c '^{' || echo 0)

  if [ -n "$matches" ] && [ "$match_count" -gt 0 ]; then
    echo "$matches" >> "$TRACKABLE_FILE"
    total_trackable=$((total_trackable + match_count))
  fi

  total_scanned=$((total_scanned + batch_count))
  CURRENT_OFFSET=$((CURRENT_OFFSET + BATCH_SIZE))

  # Update progress
  echo "{\"current_offset\": $CURRENT_OFFSET, \"total_scanned\": $total_scanned, \"total_trackable\": $total_trackable}" > "$PROGRESS_FILE"

  # Progress output every 10 batches
  if [ $((total_scanned % 1000)) -eq 0 ]; then
    echo "Progress: scanned $total_scanned, found $total_trackable trackable ($(echo "scale=2; $total_trackable * 100 / $total_scanned" | bc)%)"
  fi

  sleep $SLEEP_BETWEEN
done

echo ""
echo "=== Scan Complete ==="
echo "Total scanned: $total_scanned"
echo "Total trackable: $total_trackable"
echo "Rate: $(echo "scale=4; $total_trackable * 100 / $total_scanned" | bc)%"
echo "Output: $TRACKABLE_FILE"
