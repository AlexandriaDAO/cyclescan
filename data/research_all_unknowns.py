#!/usr/bin/env python3
"""
Research ALL canisters marked as Unknown.
This is a continuation of the batch research to properly identify all remaining unknowns.
"""

import json
import subprocess
import time
from datetime import datetime

BACKEND = "vohji-riaaa-aaaac-babxq-cai"
PROGRESS_FILE = "/home/theseus/alexandria/cyclescan/data/research_progress2.json"
RESULTS_IDENTIFIED = "/home/theseus/alexandria/cyclescan/data/research_results3.json"
RESULTS_UNKNOWN = "/home/theseus/alexandria/cyclescan/data/research_results_unknown2.json"

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
            return True, result.stdout[:500]
        return False, None
    except Exception as e:
        return False, None

def query_icrc1(canister_id):
    """Query ICRC-1 token methods."""
    try:
        result_name = subprocess.run(
            ['dfx', 'canister', '--network', 'ic', 'call', canister_id, 'icrc1_name', '()'],
            capture_output=True,
            text=True,
            timeout=30
        )
        result_symbol = subprocess.run(
            ['dfx', 'canister', '--network', 'ic', 'call', canister_id, 'icrc1_symbol', '()'],
            capture_output=True,
            text=True,
            timeout=30
        )

        if '("' in result_name.stdout and '("' in result_symbol.stdout:
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
        return False

def identify_token(name, symbol):
    """Identify token project based on name/symbol patterns."""
    if symbol.endswith("•ODIN"):
        return "ODIN.fun", "token"
    if symbol.startswith("ck"):
        return f"{symbol} Ledger", "nns_infrastructure"
    return None, "token"

def research_canister(canister_id, rank):
    """Research a single canister."""
    result = {
        "canister_id": canister_id,
        "burn_rank": rank,
        "last_researched": datetime.now().strftime("%Y-%m-%d")
    }

    # Query Candid
    candid_available, candid_data = query_candid(canister_id)
    result["candid_available"] = candid_available
    if candid_available:
        result["candid_summary"] = candid_data

    # Try ICRC-1
    is_token, token_name, token_symbol = query_icrc1(canister_id)
    if is_token:
        result["is_token"] = True
        result["token_name"] = token_name
        result["token_symbol"] = token_symbol

        project, category = identify_token(token_name, token_symbol)
        if project:
            result["project"] = project
            result["category"] = category
            result["notes"] = f"Auto-identified {token_symbol} token"
            set_project_backend(canister_id, project)
            return "identified", result
        else:
            result["project"] = None
            result["reason"] = "token_unknown_project"
            result["notes"] = f"ICRC-1 token {token_name} ({token_symbol}) - project not identified"
            return "unknown", result

    # Try Index
    is_index, ledger_id = query_index(canister_id)
    if is_index:
        is_ledger_token, ledger_name, ledger_symbol = query_icrc1(ledger_id)
        if is_ledger_token:
            project = f"{ledger_symbol} Index"
            result["project"] = project
            result["category"] = "nns_infrastructure"
            result["ledger_id"] = ledger_id
            result["notes"] = f"Index for {ledger_symbol} ledger"
            set_project_backend(canister_id, project)
            return "identified", result

    # Unknown
    result["project"] = None
    result["reason"] = "candid_unrecognizable" if candid_available else "no_candid_metadata"
    result["methods_tried"] = ["candid:service", "icrc1_name", "icrc1_symbol", "ledger_id"]
    result["notes"] = "Auto-identification failed - needs manual research"
    return "unknown", result

def main():
    """Main research loop."""
    # Load canisters
    with open('/home/theseus/alexandria/cyclescan/data/unknowns_need_proper_research.json', 'r') as f:
        canisters = json.load(f)

    print(f"Starting research for {len(canisters)} unknown canisters")

    progress = load_progress()
    progress["total_canisters"] = len(canisters)

    identified_results = load_results(RESULTS_IDENTIFIED)
    unknown_results = load_results(RESULTS_UNKNOWN)

    newly_identified = 0
    newly_unknown = 0

    for i, canister in enumerate(canisters, 1):
        canister_id = canister['canister_id']

        try:
            status, result = research_canister(canister_id, i)

            if status == "identified":
                identified_results.append(result)
                newly_identified += 1
                progress["identified"] += 1
                print(f"  [{i}/{len(canisters)}] ✓ {canister_id} -> {result['project']}")
            else:
                unknown_results.append(result)
                newly_unknown += 1
                progress["unknown_documented"] += 1
                print(f"  [{i}/{len(canisters)}] ❌ {canister_id} -> {result['reason']}")

            progress["researched"] += 1
            progress["last_canister_id"] = canister_id
            progress["remaining"] = len(canisters) - i

            if i % 50 == 0:
                save_results(RESULTS_IDENTIFIED, identified_results)
                save_results(RESULTS_UNKNOWN, unknown_results)
                save_progress(progress)
                print(f"\n  💾 Progress saved: {i}/{len(canisters)} ({newly_identified} identified, {newly_unknown} unknown)\n")

            time.sleep(0.5)  # Rate limiting

        except KeyboardInterrupt:
            print("\n\n⚠ Interrupted. Saving progress...")
            save_results(RESULTS_IDENTIFIED, identified_results)
            save_results(RESULTS_UNKNOWN, unknown_results)
            save_progress(progress)
            break
        except Exception as e:
            print(f"  ❌ Error: {e}")
            continue

    # Final save
    save_results(RESULTS_IDENTIFIED, identified_results)
    save_results(RESULTS_UNKNOWN, unknown_results)
    save_progress(progress)

    print(f"\n{'='*60}")
    print(f"Research Complete!")
    print(f"{'='*60}")
    print(f"  Researched: {len(canisters)}")
    print(f"  Identified: {newly_identified}")
    print(f"  Unknown: {newly_unknown}")

if __name__ == '__main__':
    main()
