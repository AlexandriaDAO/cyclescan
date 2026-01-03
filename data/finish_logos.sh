#!/bin/bash
LOGO_DIR="../src/cyclescan_frontend/static/logos"

fetch() {
    local url="$1"
    local name="$2"
    [ -f "${LOGO_DIR}/${name}.png" ] && echo "Skip: $name" && return 0
    curl -L -s "$url" -o "/tmp/${name}_tmp" && \
    convert "/tmp/${name}_tmp" -resize 128x128 -background white -alpha remove -alpha off \
        "${LOGO_DIR}/${name}.png" 2>/dev/null && \
    echo "✓ $name" && rm -f "/tmp/${name}_tmp" && return 0
    rm -f "/tmp/${name}_tmp"; return 1
}

# Get ICP logo (PNG version)
ICP_LOGO="https://cryptologos.cc/logos/internet-computer-icp-logo.png?v=029"
ETH_LOGO="https://cryptologos.cc/logos/ethereum-eth-logo.png?v=029"

# ICP Infrastructure - all get ICP logo
fetch "$ICP_LOGO" "evm-rpc"
fetch "$ICP_LOGO" "sns-swap"
fetch "$ICP_LOGO" "icp-index"
fetch "$ICP_LOGO" "icp-ledger"
fetch "$ICP_LOGO" "icrc-index"
fetch "$ICP_LOGO" "cycles-ledger"
fetch "$ICP_LOGO" "genesis-token"
fetch "$ICP_LOGO" "sns-component"
fetch "$ICP_LOGO" "icp-ledger-archive"
fetch "$ICP_LOGO" "bitcoin-integration"
fetch "$ICP_LOGO" "exchange-rates-oracle"

# CMC (already exists as cycles-minting-canister.png, might need rename check)
fetch "$ICP_LOGO" "cycles-minting-canister-cmc"

# 5000 SLICES
fetch "https://www.google.com/s2/favicons?domain=3npnb-hqaaa-aaaao-a2ghq-cai.icp0.io&sz=256" "5000-slices-token"

echo ""
echo "Total: $(ls -1 ${LOGO_DIR}/*.png | wc -l) logos"
