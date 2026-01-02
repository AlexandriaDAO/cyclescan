#!/bin/bash
# Quick manual favicon fetcher for projects with clear websites
# Usage: ./quick_fetch_favicons.sh

LOGO_DIR="../src/cyclescan_frontend/static/logos"

# Ensure ImageMagick is available
if ! command -v convert &> /dev/null; then
    echo "ERROR: ImageMagick not installed. Install with:"
    echo "  sudo dnf install ImageMagick  # Fedora"
    echo "  sudo apt install imagemagick  # Ubuntu"
    exit 1
fi

# Function to fetch and convert favicon
fetch_favicon() {
    local url="$1"
    local filename="$2"
    local temp_file="/tmp/${filename}_temp"

    echo "Fetching: $filename from $url"

    # Try to download
    curl -L -s -o "$temp_file" \
        -H "User-Agent: Mozilla/5.0 (compatible; CycleScan/1.0)" \
        "$url" || return 1

    # Check if file was downloaded and has content
    if [ ! -s "$temp_file" ]; then
        rm -f "$temp_file"
        return 1
    fi

    # Convert to PNG
    if convert "$temp_file" -resize 128x128 -background white -alpha remove -alpha off \
        "${LOGO_DIR}/${filename}.png" 2>/dev/null; then
        echo "  ✓ Saved ${filename}.png"
        rm -f "$temp_file"
        return 0
    else
        echo "  ✗ Failed to convert"
        rm -f "$temp_file"
        return 1
    fi
}

# Try multiple favicon sources for a domain
fetch_logo() {
    local domain="$1"
    local filename="$2"

    # Skip if already exists
    if [ -f "${LOGO_DIR}/${filename}.png" ]; then
        echo "Skip: $filename (already exists)"
        return 0
    fi

    # Try common favicon locations
    fetch_favicon "https://${domain}/favicon.ico" "$filename" && return 0
    fetch_favicon "https://${domain}/favicon.png" "$filename" && return 0
    fetch_favicon "https://${domain}/apple-touch-icon.png" "$filename" && return 0
    fetch_favicon "https://${domain}/android-chrome-192x192.png" "$filename" && return 0

    # Try Google's favicon service as fallback
    fetch_favicon "https://www.google.com/s2/favicons?domain=${domain}&sz=128" "$filename" && return 0

    echo "  ✗ Failed to fetch logo for $filename"
    return 1
}

echo "Quick Favicon Fetcher"
echo "====================="
echo ""

# High-priority projects with clear websites
fetch_logo "oisy.com" "oisy"
fetch_logo "alice.fun" "alice"
fetch_logo "kinic.io" "kinic"
fetch_logo "taggr.link" "taggr"
fetch_logo "hotornot.wtf" "dolr-ai"
fetch_logo "icpunks.com" "icpunks"
fetch_logo "omnibtc.finance" "omnibtc"
fetch_logo "yuku.app" "yuku-ai"
fetch_logo "fomowell.com" "fomowell"
fetch_logo "odin.fun" "odin-fun"
fetch_logo "bob.fun" "bob-token"
fetch_logo "catalyze.one" "catalyzedao"
fetch_logo "waterneuron.fi" "waterneuron"
fetch_logo "windoge98.com" "windoge98-token"
fetch_logo "identity.ic0.app" "internet-identity"
fetch_logo "nns.ic0.app" "nns-dapp"
fetch_logo "cycles-transfer-station.com" "cycles-transfer-station"

# ICP-specific (may need manual download)
echo ""
echo "Manual work needed for:"
echo "  - DEX (ICPSwap) - icpswap.com"
echo "  - NFT (Entrepot) - entrepot.app"
echo "  - Motoko, Neutrinite, etc."
echo ""
echo "Done! Check ${LOGO_DIR}/"
