#!/bin/bash
# Aggressive logo fetcher - tries multiple sources for better quality
# Focuses on high-priority ICP projects

LOGO_DIR="../src/cyclescan_frontend/static/logos"
cd "$(dirname "$0")"

# Check ImageMagick
if ! command -v convert &> /dev/null; then
    echo "ERROR: ImageMagick required"
    exit 1
fi

fetch_and_save() {
    local url="$1"
    local filename="$2"
    local temp_file="/tmp/${filename}_download"

    if [ -f "${LOGO_DIR}/${filename}.png" ]; then
        echo "Skip: $filename (exists)"
        return 0
    fi

    echo "Fetching $filename..."
    if curl -L -s -o "$temp_file" -H "User-Agent: Mozilla/5.0" "$url" && [ -s "$temp_file" ]; then
        if convert "$temp_file" -resize 128x128 -background white -alpha remove -alpha off \
            "${LOGO_DIR}/${filename}.png" 2>/dev/null; then
            echo "  ✓ $filename.png"
            rm -f "$temp_file"
            return 0
        fi
    fi
    rm -f "$temp_file"
    return 1
}

# Try to extract best logo from HTML page
fetch_from_page() {
    local domain="$1"
    local filename="$2"
    local html_file="/tmp/${filename}_page.html"

    if [ -f "${LOGO_DIR}/${filename}.png" ]; then
        echo "Skip: $filename (exists)"
        return 0
    fi

    echo "Parsing $domain for best logo..."

    # Download the homepage
    curl -L -s "https://${domain}" -o "$html_file" -H "User-Agent: Mozilla/5.0" 2>/dev/null || return 1

    # Extract various logo URLs from HTML
    local logo_urls=$(grep -oP '(href|src)="[^"]*\.(png|jpg|jpeg|svg|ico)"' "$html_file" | \
                     grep -i 'logo\|icon\|brand' | \
                     sed 's/.*"\([^"]*\)"/\1/' | head -5)

    # Try each discovered URL
    while IFS= read -r url; do
        [ -z "$url" ] && continue

        # Make absolute URL
        if [[ "$url" == /* ]]; then
            url="https://${domain}${url}"
        elif [[ "$url" != http* ]]; then
            url="https://${domain}/${url}"
        fi

        fetch_and_save "$url" "$filename" && rm -f "$html_file" && return 0
    done <<< "$logo_urls"

    rm -f "$html_file"
    return 1
}

echo "=== High-Priority Logo Fetcher ==="
echo ""

# ICPSwap (DEX)
fetch_and_save "https://raw.githubusercontent.com/ICPSwap-Labs/token-list/main/icpswap.png" "dex" || \
fetch_and_save "https://www.icpswap.com/logo.png" "dex" || \
fetch_and_save "https://www.icpswap.com/favicon.ico" "dex" || \
fetch_from_page "www.icpswap.com" "dex"

# Entrepot (NFT)
fetch_and_save "https://entrepot.app/logo512.png" "nft" || \
fetch_and_save "https://entrepot.app/logo192.png" "nft" || \
fetch_from_page "entrepot.app" "nft"

# SNS-W
fetch_and_save "https://dashboard.internetcomputer.org/_next/image?url=%2Fimg%2Flogo_new.svg&w=64&q=75" "sns-w" || \
fetch_and_save "https://www.google.com/s2/favicons?domain=dashboard.internetcomputer.org&sz=128" "sns-w"

# GOLDAO (if missing)
fetch_and_save "https://docs.gold-dao.org/img/logo.svg" "goldao" || \
fetch_from_page "docs.gold-dao.org" "goldao"

# Motoko
fetch_and_save "https://avatars.githubusercontent.com/u/11004800?s=200&v=4" "motoko" || \
fetch_from_page "entrepot.app" "motoko"

# OHSHII
fetch_from_page "launcher.ohshii.com" "ohshii"

# Omnity
fetch_from_page "www.omnity.network" "omnity"

# TipJar
fetch_and_save "https://k25co-pqaaa-aaaab-aaakq-cai.ic0.app/favicon.ico" "tipjar"

# EVM RPC (use ICP logo)
fetch_and_save "https://dashboard.internetcomputer.org/_next/image?url=%2Fimg%2Flogo_new.svg&w=64&q=75" "evm-rpc"

# RunicSwap
fetch_and_save "https://github.com/buriburizaemonnn.png" "runicswap"

# Ordi Trade (SNS)
fetch_and_save "https://www.google.com/s2/favicons?domain=dashboard.internetcomputer.org&sz=128" "ordi-trade"

# GRAVE
fetch_and_save "https://xc3mi-sqaaa-aaaaj-a2mhq-cai.icp0.io/favicon.ico" "grave-token"

# NNS components (use ICP logo)
for name in "nns-root" "nns-lifeline" "nns-registry" "nns-governance" "nns-subnet-rental" "nns-subnet-management" "nns-canister-migration" "nns-node-provider-rewards"; do
    fetch_and_save "https://dashboard.internetcomputer.org/favicon.ico" "$name" || \
    fetch_and_save "https://www.google.com/s2/favicons?domain=internetcomputer.org&sz=128" "$name"
done

# ckERC20 tokens (use ICP chain fusion logo)
for name in "cketh-index" "cketh-ledger" "cketh-minter" "ckuni-index" "ckeurc-index" "cklink-index" "cklink-ledger" "ckoct-ledger" "ckpepe-index" "ckusdc-index" "ckusdc-ledger" "ckxaut-index" "ckerc20-orchestrator"; do
    fetch_and_save "https://internetcomputer.org/img/favicon/favicon-32x32.png" "$name"
done

# ICP Index/Ledger (use ICP logo)
fetch_and_save "https://dashboard.internetcomputer.org/favicon.ico" "icp-index"
fetch_and_save "https://dashboard.internetcomputer.org/favicon.ico" "icp-ledger"
fetch_and_save "https://dashboard.internetcomputer.org/favicon.ico" "icp-ledger-archive"

# ICRC Index
fetch_and_save "https://internetcomputer.org/img/favicon/favicon-32x32.png" "icrc-index"

# CanDB
fetch_from_page "www.canscale.dev" "candb-storage"

# Cycles Ledger
fetch_and_save "https://internetcomputer.org/img/favicon/favicon-32x32.png" "cycles-ledger"

# Genesis Token (NNS)
fetch_and_save "https://dashboard.internetcomputer.org/favicon.ico" "genesis-token"

# Orbit Station
fetch_from_page "orbit.global" "orbit-station"

# SNS Component
fetch_and_save "https://internetcomputer.org/img/favicon/favicon-32x32.png" "sns-component"
fetch_and_save "https://internetcomputer.org/img/favicon/favicon-32x32.png" "sns-swap"

# Bitcoin Integration
fetch_and_save "https://internetcomputer.org/img/favicon/favicon-32x32.png" "bitcoin-integration"

# 5000 SLICES
fetch_and_save "https://3npnb-hqaaa-aaaao-a2ghq-cai.icp0.io/favicon.ico" "5000-slices-token"

# Exchange Rates Oracle
fetch_and_save "https://internetcomputer.org/img/favicon/favicon-32x32.png" "exchange-rates-oracle"

# Avocado Research
fetch_from_page "avcd.hodl.fyi" "avocado-research-token"

# The Needful DO
fetch_and_save "https://github.com/The-Needful-DO-org.png" "the-needful-do-token"

echo ""
echo "=== Summary ==="
total=$(ls -1 "${LOGO_DIR}"/*.png 2>/dev/null | wc -l)
echo "Total logos now: $total"
echo "Location: ${LOGO_DIR}/"
