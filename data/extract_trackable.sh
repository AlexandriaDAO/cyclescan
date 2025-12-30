#!/bin/bash
# Extract trackable canisters (those with blackhole controllers) from raw data
# Usage: ./extract_trackable.sh canisters_0_25000.json [output.json]

set -e

INPUT_FILE="${1:?Usage: $0 <input.json> [output.json]}"
OUTPUT_FILE="${2:-trackable_$(basename "$INPUT_FILE")}"

# Blackhole controller IDs
BLACKHOLES=(
  "e3mmv-5qaaa-aaaah-aadma-cai"  # ninegua Original
  "5vdms-kaaaa-aaaap-aa3uq-cai"  # CycleOps V1
  "2daxo-giaaa-aaaap-anvca-cai"  # CycleOps V2
  "cpbhu-5iaaa-aaaad-aalta-cai"  # CycleOps V3
  "w7sux-siaaa-aaaai-qpasa-cai"  # Cygnus
  "r7inp-6aaaa-aaaaa-aaabq-cai"  # NNS Root
)

# Build pattern for matching
PATTERN=$(IFS='|'; echo "${BLACKHOLES[*]}")

echo "Processing: $INPUT_FILE"
echo "Output: $OUTPUT_FILE"
echo ""

# Extract trackable canisters with cleaned fields and identify proxy
jq --arg pattern "$PATTERN" --argjson blackholes "$(printf '%s\n' "${BLACKHOLES[@]}" | jq -R . | jq -s .)" '
[
  .[] |
  # Find which blackhole controllers this canister has
  . as $canister |
  ($blackholes | map(select(. as $bh | $canister.controllers | index($bh)))) as $matching_proxies |

  # Only include if at least one blackhole controller found
  select(($matching_proxies | length) > 0) |

  # Build cleaned record
  {
    canister_id: .canister_id,
    proxy: $matching_proxies[0],           # Primary proxy to use
    all_proxies: $matching_proxies,        # All matching blackholes
    controllers: .controllers,
    name: (.name // ""),
    language: (.language // ""),
    module_hash: (.module_hash // "")
  }
]
' "$INPUT_FILE" > "$OUTPUT_FILE"

# Stats
total_input=$(jq 'length' "$INPUT_FILE")
total_output=$(jq 'length' "$OUTPUT_FILE")
rate=$(echo "scale=2; $total_output * 100 / $total_input" | bc)

echo "=== Results ==="
echo "Input canisters:  $total_input"
echo "Trackable found:  $total_output"
echo "Rate:             ${rate}%"
echo ""
echo "Output saved to: $OUTPUT_FILE"
