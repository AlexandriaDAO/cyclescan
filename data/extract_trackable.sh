#!/bin/bash
# Extract publicly queryable canisters from raw IC Dashboard data
# Only keeps canisters with ninegua or NNS Root as controller
#
# Usage: ./extract_public.sh canisters_0_25000.json [output.json]

set -e

INPUT_FILE="${1:?Usage: $0 <input.json> [output.json]}"
OUTPUT_FILE="${2:-public_$(basename "$INPUT_FILE")}"

# Public blackhole controllers (can query canister_status)
# - ninegua: Original blackhole, anyone can query
# - NNS Root: System canister, used by SNS projects
NINEGUA="e3mmv-5qaaa-aaaah-aadma-cai"
NNS_ROOT="r7inp-6aaaa-aaaaa-aaabq-cai"

echo "Input:  $INPUT_FILE"
echo "Output: $OUTPUT_FILE"
echo ""

# Extract canisters with public blackhole controllers
# Priority: ninegua > NNS Root (ninegua is simpler/faster)
jq --arg ninegua "$NINEGUA" --arg nns "$NNS_ROOT" '
[
  .[] |

  # Check which public proxies this canister has
  (if (.controllers | index($ninegua)) then $ninegua
   elif (.controllers | index($nns)) then $nns
   else null end) as $proxy |

  # Only include if we found a public proxy
  select($proxy != null) |

  # Output minimal record
  {
    canister_id: .canister_id,
    proxy_id: $proxy
  }
]
' "$INPUT_FILE" > "$OUTPUT_FILE"

# Stats
total_input=$(jq 'length' "$INPUT_FILE")
total_output=$(jq 'length' "$OUTPUT_FILE")

echo "=== Results ==="
echo "Input canisters:  $total_input"
echo "Public canisters: $total_output"
echo ""

# Breakdown by proxy
echo "By proxy:"
jq -r 'group_by(.proxy_id) | .[] | "  \(.[0].proxy_id): \(length)"' "$OUTPUT_FILE"
