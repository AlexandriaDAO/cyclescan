# Logo Fetching Guide

This guide explains how to fetch and add logos for all CycleScan projects.

## Current Status

- **36 logos** exist out of 155 projects (~23% coverage)
- Logos are stored as static PNG files in `/src/cyclescan_frontend/static/logos/`
- Naming convention: kebab-case (e.g., `boom.png`, `sonic.png`)

## Logo Sources

### 1. ICRC-1 Token Metadata (SVG)
Many ICP tokens store their logos in ICRC-1 metadata as base64-encoded SVGs.

**Example query:**
```bash
dfx canister --network ic call <token_canister_id> icrc1_metadata '()'
```

Look for the `icrc1:logo` field containing `data:image/svg+xml;base64,...`

### 2. Website Favicons
For projects with websites, download their favicon/logo from:
- `{website}/favicon.ico`
- `{website}/favicon.png`
- `{website}/apple-touch-icon.png`
- Parse HTML `<link rel="icon">` tags

### 3. Manual Curation
Some logos may need to be:
- Searched on Google Images
- Extracted from project documentation
- Created as simple text logos
- Found on ICP ecosystem sites (dashboard.internetcomputer.org)

## Automated Fetching

### Prerequisites

Install dependencies:
```bash
pip install Pillow requests

# Optional (for better SVG conversion):
pip install cairosvg

# Or use ImageMagick:
sudo dnf install ImageMagick  # Fedora
sudo apt install imagemagick  # Ubuntu
```

### Run the Fetcher

```bash
cd data
python3 fetch_logos.py
```

This script will:
1. Check which projects are missing logos
2. Try to fetch from ICRC-1 metadata (for tokens)
3. Try to fetch from website favicons
4. Convert all to 128x128 PNG format
5. Skip projects that already have logos

### Check Progress

```bash
python3 check_missing_logos.py
```

Outputs:
- Summary of coverage
- List of missing logos with websites
- CSV file for manual tracking

## Manual Logo Addition

For projects where automation fails:

1. **Find the logo** (Google, project website, ICP Dashboard)

2. **Convert to PNG** (128x128 recommended):
   ```bash
   convert input.svg -resize 128x128 output.png
   # or
   convert input.ico -resize 128x128 output.png
   ```

3. **Name correctly** using kebab-case:
   ```bash
   # Project: "ICPSwap Token" → filename: icpswap-token.png
   # Project: "Cycles Minting Canister (CMC)" → filename: cycles-minting-canister-cmc.png
   ```

4. **Save to**: `/src/cyclescan_frontend/static/logos/{filename}.png`

5. **Test** by deploying and checking the dashboard

## Naming Convention Reference

The frontend transforms project names like this (from `+page.svelte:68-77`):

```javascript
const filename = project
  .toLowerCase()
  .replace(/[^a-z0-9]/g, '-')  // Replace non-alphanumeric with dash
  .replace(/-+/g, '-')          // Collapse multiple dashes
  .replace(/^-|-$/g, '');       // Remove leading/trailing dashes
```

**Examples:**
- `"BOOM"` → `boom.png`
- `"OpenChat"` → `openchat.png`
- `"Cycles Minting Canister (CMC)"` → `cycles-minting-canister-cmc.png`
- `"Cecil The Lion DAO"` → `cecil-the-lion-dao.png`

## Helpful Resources

### ICP Token Lists
- SNS tokens: Query via `get_sns_canisters_summary()`
- ICRC tokens: Many listed on ICPCoins, ICPSwap, etc.

### Logo Sources
- [ICP Dashboard](https://dashboard.internetcomputer.org/) - Official logos for SNS projects
- [ICPCoins](https://icpcoins.com/) - Token logos
- [Dank's IC Assets](https://github.com/DankGang/ic-assets) - Community logo collection

### Tools
- [Favicon Grabber](https://www.google.com/s2/favicons?domain=example.com) - Google's favicon service
- [RealFaviconGenerator](https://realfavicongenerator.net/) - Multi-size favicon tool

## Priority Projects

Focus on high-burn projects first (these appear at top of leaderboard):

1. Projects with websites but no logos
2. SNS projects (have official branding)
3. Popular tokens (CHAT, DEX, NFT platforms)
4. Infrastructure (NNS, CMC, etc.)

## After Adding Logos

1. **Test locally**: Deploy and check `http://localhost:4943/` (if using local replica)
2. **Deploy to mainnet**: Run `./scripts/deploy.sh`
3. **Verify**: Check https://vohji-riaaa-aaaac-babxq-cai.icp0.io

No backend changes needed - logos are purely frontend assets!
