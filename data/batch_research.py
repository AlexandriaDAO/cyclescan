#!/usr/bin/env python3
"""
Batch research all unknown canisters following the playbook.
Processes each canister, identifies it or documents why identification failed.
"""

import json
import subprocess
import sys
import time
from datetime import datetime

BACKEND = "vohji-riaaa-aaaac-babxq-cai"
PROGRESS_FILE = "/home/theseus/alexandria/cyclescan/data/research_progress.json"
RESULTS_IDENTIFIED = "/home/theseus/alexandria/cyclescan/data/research_results2.json"
RESULTS_UNKNOWN = "/home/theseus/alexandria/cyclescan/data/research_results_unknown.json"

def load_progress():
    """Load progress tracker."""
    try:
        with open(PROGRESS_FILE, 'r') as f:
            return json.load(f)
    except:
        return {
            "total_canisters": 0,
            "researched": 0,
            "identified": 0,
            "unknown_documented": 0,
            "remaining": 0,
            "last_canister_id": None,
            "last_updated": None
        }

def save_progress(progress):
    """Save progress tracker."""
    progress["last_updated"] = datetime.now().isoformat()
    with open(PROGRESS_FILE, 'w') as f:
        json.dump(progress, f, indent=2)

def load_results(filename):
    """Load results file."""
    try:
        with open(filename, 'r') as f:
            return json.load(f)
    except:
        return []

def save_results(filename, results):
    """Save results file."""
    with open(filename, 'w') as f:
        json.dump(results, f, indent=2)

def query_candid(canister_id):
    """Query Candid metadata."""
    try:
        result = subprocess.run(
            ['dfx', 'canister', '--network', 'ic', 'metadata', canister_id, 'candid:service'],
            capture_output=True,
            text=True,
            timeout=30
        )
        if result.returncode == 0:
            return True, result.stdout[:500]  # First 500 chars
        return False, None
    except Exception as e:
        return False, str(e)

def query_icrc1(canister_id):
    """Query ICRC-1 token methods."""
    try:
        # Query name
        result_name = subprocess.run(
            ['dfx', 'canister', '--network', 'ic', 'call', canister_id, 'icrc1_name', '()'],
            capture_output=True,
            text=True,
            timeout=30
        )

        # Query symbol
        result_symbol = subprocess.run(
            ['dfx', 'canister', '--network', 'ic', 'call', canister_id, 'icrc1_symbol', '()'],
            capture_output=True,
            text=True,
            timeout=30
        )

        if '("' in result_name.stdout and '("' in result_symbol.stdout:
            # Extract name and symbol
            name = result_name.stdout.split('("')[1].split('"')[0] if '("' in result_name.stdout else None
            symbol = result_symbol.stdout.split('("')[1].split('"')[0] if '("' in result_symbol.stdout else None
            return True, name, symbol
        return False, None, None
    except Exception as e:
        return False, None, None

def query_index(canister_id):
    """Query index canister ledger_id method."""
    try:
        result = subprocess.run(
            ['dfx', 'canister', '--network', 'ic', 'call', canister_id, 'ledger_id', '()'],
            capture_output=True,
            text=True,
            timeout=30
        )

        if 'principal "' in result.stdout:
            ledger_id = result.stdout.split('principal "')[1].split('"')[0]
            return True, ledger_id
        return False, None
    except Exception as e:
        return False, None

def set_project_backend(canister_id, project_name):
    """Set project name in backend."""
    try:
        cmd = [
            'dfx', 'canister', '--network', 'ic', 'call',
            BACKEND, 'set_project',
            f'(principal "{canister_id}", opt "{project_name}")'
        ]
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=60)
        return result.returncode == 0
    except Exception as e:
        print(f"  ⚠ Error setting project: {e}")
        return False

def identify_token(name, symbol):
    """Identify token project based on name/symbol patterns."""
    # ODIN pattern
    if symbol.endswith("•ODIN"):
        return "ODIN.fun", "token"

    # ck* pattern
    if symbol.startswith("ck"):
        return f"{symbol} Ledger", "nns_infrastructure"

    # Unknown token
    return None, "token"

def research_canister(canister_id, rank):
    """Research a single canister following the playbook."""
    print(f"\n{'='*60}")
    print(f"Researching [{rank}]: {canister_id}")
    print(f"{'='*60}")

    result = {
        "canister_id": canister_id,
        "burn_rank": rank,
        "last_researched": datetime.now().strftime("%Y-%m-%d")
    }

    # Step 1: Query Candid
    print("  [1/3] Querying Candid metadata...")
    candid_available, candid_data = query_candid(canister_id)
    result["candid_available"] = candid_available

    if candid_available:
        print(f"    ✓ Candid available")
        result["candid_summary"] = candid_data
    else:
        print(f"    ❌ No Candid metadata")

    # Step 2: Try ICRC-1
    print("  [2/3] Trying ICRC-1 token methods...")
    is_token, token_name, token_symbol = query_icrc1(canister_id)

    if is_token:
        print(f"    ✓ ICRC-1 Token: {token_name} ({token_symbol})")
        result["is_token"] = True
        result["token_name"] = token_name
        result["token_symbol"] = token_symbol

        # Identify project
        project, category = identify_token(token_name, token_symbol)
        if project:
            print(f"    ✓ Identified as: {project}")
            result["project"] = project
            result["category"] = category
            result["notes"] = f"Auto-identified {token_symbol} token"

            # Set in backend
            if set_project_backend(canister_id, project):
                print(f"    ✓ Project set in backend")
                return "identified", result
            else:
                print(f"    ⚠ Failed to set in backend, will retry")
                return "identified", result
        else:
            print(f"    ⚠ Token found but project unknown")
            result["project"] = None
            result["reason"] = "token_unknown_project"
            result["notes"] = f"ICRC-1 token {token_name} ({token_symbol}) - project not identified"
            return "unknown", result
    else:
        print(f"    ❌ Not an ICRC-1 token")

    # Step 3: Try Index canister
    print("  [3/3] Trying index canister methods...")
    is_index, ledger_id = query_index(canister_id)

    if is_index:
        print(f"    ✓ Index canister! Ledger: {ledger_id}")

        # Query ledger for token info
        is_ledger_token, ledger_name, ledger_symbol = query_icrc1(ledger_id)
        if is_ledger_token:
            project = f"{ledger_symbol} Index"
            print(f"    ✓ Identified as: {project}")
            result["project"] = project
            result["category"] = "nns_infrastructure"
            result["ledger_id"] = ledger_id
            result["notes"] = f"Index for {ledger_symbol} ledger"

            # Set in backend
            if set_project_backend(canister_id, project):
                print(f"    ✓ Project set in backend")
                return "identified", result
            else:
                print(f"    ⚠ Failed to set in backend, will retry")
                return "identified", result
        else:
            print(f"    ⚠ Index found but ledger token info unavailable")
            result["project"] = None
            result["reason"] = "index_ledger_unknown"
            result["ledger_id"] = ledger_id
            result["notes"] = f"Index canister for {ledger_id} but could not identify token"
            return "unknown", result
    else:
        print(f"    ❌ Not an index canister")

    # Could not identify
    print(f"  ❌ Unable to auto-identify")
    result["project"] = None
    result["reason"] = "candid_unrecognizable" if candid_available else "no_candid_metadata"
    result["methods_tried"] = ["candid:service", "icrc1_name", "icrc1_symbol", "ledger_id"]
    result["notes"] = "Auto-identification failed - needs manual research"

    return "unknown", result

def main():
    """Main research loop."""
    # Load canisters to research
    with open('/home/theseus/alexandria/cyclescan/data/needs_research.json', 'r') as f:
        canisters = json.load(f)

    print(f"Starting batch research for {len(canisters)} canisters")

    # Load progress
    progress = load_progress()
    progress["total_canisters"] = len(canisters)

    # Load existing results
    identified_results = load_results(RESULTS_IDENTIFIED)
    unknown_results = load_results(RESULTS_UNKNOWN)

    # Track stats
    newly_identified = 0
    newly_unknown = 0

    # Process each canister
    for i, canister in enumerate(canisters, 1):
        canister_id = canister['canister_id']

        try:
            status, result = research_canister(canister_id, i)

            if status == "identified":
                identified_results.append(result)
                newly_identified += 1
                progress["identified"] += 1
            else:
                unknown_results.append(result)
                newly_unknown += 1
                progress["unknown_documented"] += 1

            progress["researched"] += 1
            progress["last_canister_id"] = canister_id
            progress["remaining"] = len(canisters) - i

            # Save results every 10 canisters
            if i % 10 == 0:
                save_results(RESULTS_IDENTIFIED, identified_results)
                save_results(RESULTS_UNKNOWN, unknown_results)
                save_progress(progress)
                print(f"\n  💾 Progress saved: {i}/{len(canisters)} ({newly_identified} identified, {newly_unknown} unknown)")

            # Rate limiting
            time.sleep(1)

        except KeyboardInterrupt:
            print("\n\n⚠ Interrupted by user. Saving progress...")
            save_results(RESULTS_IDENTIFIED, identified_results)
            save_results(RESULTS_UNKNOWN, unknown_results)
            save_progress(progress)
            sys.exit(0)
        except Exception as e:
            print(f"\n  ❌ Error researching {canister_id}: {e}")
            # Continue with next canister
            continue

    # Final save
    save_results(RESULTS_IDENTIFIED, identified_results)
    save_results(RESULTS_UNKNOWN, unknown_results)
    save_progress(progress)

    print(f"\n{'='*60}")
    print(f"Research Complete!")
    print(f"{'='*60}")
    print(f"  Total researched: {len(canisters)}")
    print(f"  Newly identified: {newly_identified}")
    print(f"  Newly unknown: {newly_unknown}")
    print(f"  Total identified in file: {len(identified_results)}")
    print(f"  Total unknown in file: {len(unknown_results)}")
    print(f"\n  Results saved to:")
    print(f"    - {RESULTS_IDENTIFIED}")
    print(f"    - {RESULTS_UNKNOWN}")

if __name__ == '__main__':
    main()
