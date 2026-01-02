#!/bin/bash
# Final batch - remaining high-value projects
LOGO_DIR="../src/cyclescan_frontend/static/logos"
cd "$(dirname "$0")"

fetch() {
    local url="$1"
    local name="$2"
    [ -f "${LOGO_DIR}/${name}.png" ] && echo "Skip: $name" && return 0
    echo "Fetch: $name"
    curl -L -s "$url" -H "User-Agent: Mozilla/5.0" -o "/tmp/${name}" && \
    convert "/tmp/${name}" -resize 128x128 -background white -alpha remove \
        "${LOGO_DIR}/${name}.png" 2>/dev/null && \
    echo "  ✓ $name.png" && rm -f "/tmp/${name}" && return 0
    rm -f "/tmp/${name}"; return 1
}

# ICPSwap (DEX) - try direct logo
fetch "https://app.icpswap.com/images/logo.svg" "dex" || \
fetch "https://github.com/ICPSwap-Labs.png" "dex" || \
fetch "https://www.google.com/s2/favicons?domain=icpswap.com&sz=256" "dex"

# GoldDAO
fetch "https://avatars.githubusercontent.com/u/122962761?s=200" "goldao" || \
fetch "https://www.google.com/s2/favicons?domain=gold-dao.org&sz=256" "goldao"

# TipJar
fetch "https://k25co-pqaaa-aaaab-aaakq-cai.ic0.app/logo.png" "tipjar" || \
fetch "https://www.google.com/s2/favicons?domain=k25co-pqaaa-aaaab-aaakq-cai.ic0.app&sz=256" "tipjar"

# EVM RPC (ICP logo)
fetch "https://internetcomputer.org/img/IC_logo_horizontal.svg" "evm-rpc"

# GRAVE
fetch "https://xc3mi-sqaaa-aaaaj-a2mhq-cai.icp0.io/logo.png" "grave-token" || \
fetch "https://www.google.com/s2/favicons?domain=xc3mi-sqaaa-aaaaj-a2mhq-cai.icp0.io&sz=256" "grave-token"

# BoB (Burn or Burn)
fetch "https://bob.fun/logo.png" "bob-burn-or-burn" || \
fetch "https://www.google.com/s2/favicons?domain=bob.fun&sz=256" "bob-burn-or-burn"

# CanDB
fetch "https://www.canscale.dev/logo.png" "candb-storage" || \
fetch "https://www.google.com/s2/favicons?domain=canscale.dev&sz=256" "candb-storage"

# Avocado Research
fetch "https://avcd.hodl.fyi/logo.png" "avocado-research-token" || \
fetch "https://www.google.com/s2/favicons?domain=avcd.hodl.fyi&sz=256" "avocado-research-token"

# All ICP infrastructure (use ICP logo)
for name in "cycles-ledger" "genesis-token" "sns-component" "sns-swap" \
            "bitcoin-integration" "exchange-rates-oracle" "icrc-index" \
            "icp-index" "icp-ledger" "icp-ledger-archive" "evm-rpc"; do
    fetch "https://internetcomputer.org/img/IC_logo_horizontal.svg" "$name"
done

# Chain Key tokens (Ethereum bridge logo)
for name in "cketh-index" "cketh-ledger" "cketh-minter" "ckuni-index" \
            "ckeurc-index" "cklink-index" "cklink-ledger" "ckoct-ledger" \
            "ckpepe-index" "ckusdc-index" "ckusdc-ledger" "ckxaut-index"; do
    fetch "https://cryptologos.cc/logos/ethereum-eth-logo.svg?v=025" "$name" || \
    fetch "https://internetcomputer.org/img/IC_logo_horizontal.svg" "$name"
done

# 5000 SLICES
fetch "https://3npnb-hqaaa-aaaao-a2ghq-cai.icp0.io/logo.png" "5000-slices-token"

echo "Total: $(ls -1 ${LOGO_DIR}/*.png | wc -l) logos"
