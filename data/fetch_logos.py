#!/usr/bin/env python3
"""
Fetch logos for CycleScan projects from multiple sources:
1. ICRC-1 token metadata (SVG logos)
2. Website favicons
3. Manual fallbacks

Outputs PNG files to ../src/cyclescan_frontend/static/logos/
"""

import json
import requests
import base64
import re
from pathlib import Path
from typing import Optional
from urllib.parse import urlparse
import time

# Directories
DATA_DIR = Path(__file__).parent
LOGO_DIR = DATA_DIR.parent / "src/cyclescan_frontend/static/logos"
BACKUP_FILE = DATA_DIR / "backup/projects_backup.json"
CANISTERS_FILE = DATA_DIR / "backup/canisters_backup.json"

LOGO_DIR.mkdir(exist_ok=True)

def normalize_project_name(name: str) -> str:
    """Convert project name to kebab-case filename (matches frontend logic)"""
    return re.sub(r'^-|-$', '', re.sub(r'-+', '-', re.sub(r'[^a-z0-9]', '-', name.lower())))

def get_token_canister_id(project_name: str, canisters_data: list) -> Optional[str]:
    """Find a canister ID for this project (prefer tokens/ledgers)"""
    # Look for canisters belonging to this project
    for canister in canisters_data:
        if canister.get("project") and canister["project"][0] == project_name:
            canister_id = canister["canister_id"]
            # Prefer token/ledger canisters (more likely to have logos)
            if "token" in project_name.lower() or "ledger" in project_name.lower():
                return canister_id

    # If no specific match, return first canister for this project
    for canister in canisters_data:
        if canister.get("project") and canister["project"][0] == project_name:
            return canister["canister_id"]

    return None

def fetch_icrc1_logo(canister_id: str) -> Optional[str]:
    """Fetch logo from ICRC-1 token metadata (returns SVG or data URL)"""
    try:
        # Query icrc1_metadata endpoint
        response = requests.post(
            "https://ic0.app/api/v2/canister/{}/query".format(canister_id.replace("-", "")),
            headers={"Content-Type": "application/cbor"},
            timeout=10
        )

        # This is simplified - actual implementation needs CBOR encoding
        # For now, we'll use dfx calls via subprocess
        import subprocess
        result = subprocess.run(
            ["dfx", "canister", "--network", "ic", "call", canister_id, "icrc1_metadata", "()"],
            capture_output=True,
            text=True,
            timeout=15
        )

        if result.returncode == 0:
            # Parse output for logo field
            output = result.stdout
            if "icrc1:logo" in output or '"logo"' in output:
                # Extract base64 SVG data
                # Format: ("icrc1:logo", variant { Text = "data:image/svg+xml;base64,..." })
                match = re.search(r'data:image/svg\+xml;base64,([A-Za-z0-9+/=]+)', output)
                if match:
                    return match.group(0)  # Return full data URL

        return None
    except Exception as e:
        print(f"  Error fetching ICRC-1 metadata for {canister_id}: {e}")
        return None

def fetch_favicon(website_url: str) -> Optional[bytes]:
    """Fetch favicon from website (tries multiple common locations)"""
    if not website_url:
        return None

    try:
        parsed = urlparse(website_url)
        base_url = f"{parsed.scheme}://{parsed.netloc}"

        # Try multiple favicon locations
        favicon_urls = [
            f"{base_url}/favicon.ico",
            f"{base_url}/favicon.png",
            f"{base_url}/apple-touch-icon.png",
            f"{base_url}/android-chrome-192x192.png",
        ]

        # Also try to parse HTML for <link rel="icon">
        try:
            html_response = requests.get(website_url, timeout=10, headers={
                "User-Agent": "Mozilla/5.0 (compatible; CycleScan/1.0)"
            })
            if html_response.status_code == 200:
                # Simple regex to find icon links (not a full HTML parser)
                icon_matches = re.findall(
                    r'<link[^>]+rel=["\'](?:icon|shortcut icon|apple-touch-icon)["\'][^>]+href=["\'](/[^"\']+)["\']',
                    html_response.text,
                    re.IGNORECASE
                )
                for icon_path in icon_matches:
                    if icon_path.startswith('/'):
                        favicon_urls.insert(0, f"{base_url}{icon_path}")
        except:
            pass

        # Try each URL
        for url in favicon_urls:
            try:
                response = requests.get(url, timeout=10, headers={
                    "User-Agent": "Mozilla/5.0 (compatible; CycleScan/1.0)"
                })
                if response.status_code == 200 and len(response.content) > 100:
                    return response.content
            except:
                continue

        return None
    except Exception as e:
        print(f"  Error fetching favicon from {website_url}: {e}")
        return None

def svg_to_png(svg_data: str, output_path: Path, size: int = 128):
    """Convert SVG (data URL or raw) to PNG using cairosvg or imagemagick"""
    try:
        # Extract SVG content from data URL
        if svg_data.startswith("data:image/svg+xml;base64,"):
            svg_bytes = base64.b64decode(svg_data.split(",", 1)[1])
        else:
            svg_bytes = svg_data.encode()

        # Try cairosvg first (cleaner output)
        try:
            import cairosvg
            cairosvg.svg2png(
                bytestring=svg_bytes,
                write_to=str(output_path),
                output_width=size,
                output_height=size
            )
            return True
        except ImportError:
            pass

        # Fallback to ImageMagick
        import subprocess
        svg_temp = output_path.parent / f"{output_path.stem}_temp.svg"
        svg_temp.write_bytes(svg_bytes)

        result = subprocess.run(
            ["convert", "-background", "none", "-resize", f"{size}x{size}",
             str(svg_temp), str(output_path)],
            capture_output=True
        )
        svg_temp.unlink()

        return result.returncode == 0
    except Exception as e:
        print(f"  Error converting SVG to PNG: {e}")
        return False

def convert_image_to_png(image_data: bytes, output_path: Path, size: int = 128):
    """Convert any image format to PNG using PIL"""
    try:
        from PIL import Image
        import io

        img = Image.open(io.BytesIO(image_data))

        # Convert RGBA if needed
        if img.mode in ('RGBA', 'LA'):
            # Create white background
            background = Image.new('RGB', img.size, (255, 255, 255))
            if img.mode == 'RGBA':
                background.paste(img, mask=img.split()[3])
            else:
                background.paste(img, mask=img.split()[1])
            img = background
        elif img.mode != 'RGB':
            img = img.convert('RGB')

        # Resize to square
        img.thumbnail((size, size), Image.Resampling.LANCZOS)

        # Save as PNG
        img.save(output_path, "PNG")
        return True
    except Exception as e:
        print(f"  Error converting image: {e}")
        return False

def main():
    print("CycleScan Logo Fetcher")
    print("=" * 50)

    # Check dependencies
    try:
        from PIL import Image
    except ImportError:
        print("ERROR: Pillow required. Install with: pip install Pillow")
        return

    # Load project data
    with open(BACKUP_FILE) as f:
        projects = json.load(f)

    with open(CANISTERS_FILE) as f:
        canisters = json.load(f)

    print(f"Loaded {len(projects)} projects")
    print(f"Logo output directory: {LOGO_DIR}")
    print()

    stats = {"total": 0, "token_metadata": 0, "favicon": 0, "skipped": 0, "failed": 0}

    for project in projects:
        name = project["name"]
        website = project["website"][0] if project["website"] else None
        filename = normalize_project_name(name)
        output_path = LOGO_DIR / f"{filename}.png"

        stats["total"] += 1

        # Skip if logo already exists
        if output_path.exists():
            print(f"[{stats['total']}/{len(projects)}] ✓ {name} (already exists)")
            stats["skipped"] += 1
            continue

        print(f"[{stats['total']}/{len(projects)}] Fetching: {name}")
        success = False

        # Strategy 1: Try ICRC-1 token metadata
        canister_id = get_token_canister_id(name, canisters)
        if canister_id and ("token" in name.lower() or "ledger" in name.lower()):
            print(f"  Trying token metadata for {canister_id}...")
            logo_data = fetch_icrc1_logo(canister_id)
            if logo_data:
                if svg_to_png(logo_data, output_path):
                    print(f"  ✓ Saved from token metadata")
                    stats["token_metadata"] += 1
                    success = True
                    time.sleep(0.5)  # Rate limiting

        # Strategy 2: Try website favicon
        if not success and website:
            print(f"  Trying favicon from {website}...")
            favicon_data = fetch_favicon(website)
            if favicon_data:
                if convert_image_to_png(favicon_data, output_path):
                    print(f"  ✓ Saved from favicon")
                    stats["favicon"] += 1
                    success = True
                    time.sleep(0.5)

        if not success:
            print(f"  ✗ No logo found")
            stats["failed"] += 1

        time.sleep(0.2)  # Basic rate limiting

    print()
    print("=" * 50)
    print("Summary:")
    print(f"  Total projects: {stats['total']}")
    print(f"  Already had logos: {stats['skipped']}")
    print(f"  Fetched from token metadata: {stats['token_metadata']}")
    print(f"  Fetched from favicon: {stats['favicon']}")
    print(f"  Failed to fetch: {stats['failed']}")
    print()
    print(f"Logos saved to: {LOGO_DIR}")

if __name__ == "__main__":
    main()
