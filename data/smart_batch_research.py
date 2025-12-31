#!/usr/bin/env python3
"""
Smart batch research using learned patterns.
Processes canisters intelligently with pattern matching.
"""

import json
import subprocess
import time
from datetime import datetime

BACKEND = "vohji-riaaa-aaaac-babxq-cai"

def query_candid(cid):
    try:
        result = subprocess.run(['dfx', 'canister', '--network', 'ic', 'metadata', cid, 'candid:service'],
                              capture_output=True, text=True, timeout=30)
        return result.stdout if result.returncode == 0 else None
    except:
        return None

def query_icrc1(cid):
    try:
        name_result = subprocess.run(['dfx', 'canister', '--network', 'ic', 'call', cid, 'icrc1_name', '()'],
                                   capture_output=True, text=True, timeout=30)
        symbol_result = subprocess.run(['dfx', 'canister', '--network', 'ic', 'call', cid, 'icrc1_symbol', '()'],
                                      capture_output=True, text=True, timeout=30)
        if '("' in symbol_result.stdout:
            name = name_result.stdout.split('("')[1].split('"')[0] if '("' in name_result.stdout else None
            symbol = symbol_result.stdout.split('("')[1].split('"')[0]
            return name, symbol
    except:
        pass
    return None, None

def query_ledger_id(cid):
    try:
        result = subprocess.run(['dfx', 'canister', '--network', 'ic', 'call', cid, 'ledger_id', '()'],
                              capture_output=True, text=True, timeout=30)
        if 'principal "' in result.stdout:
            return result.stdout.split('principal "')[1].split('"')[0]
    except:
        pass
    return None

def identify_by_candid(candid):
    """Identify project by Candid patterns."""
    if not candid:
        return None, "no_candid_metadata"

    # Asset Canister
    if 'CreateAssetArguments' in candid and 'BatchId' in candid and 'ChunkId' in candid:
        return "Asset Canister", "infrastructure"

    # Orbit Station
    if 'RequestPolicy' in candid and 'AssetSymbol' in candid and 'NetworkId' in candid:
        return "Orbit Station", "infrastructure"

    # Omnity
    if ('hub_principal' in candid and 'GenerateTicketReq' in candid) or 'omnity_chain_id' in candid:
        return "Omnity", "defi"

    # OmniBTC
    if ('customs' in candid and 'indexer' in candid) or 'RuneBalance' in candid:
        return "OmniBTC", "defi"

    # ICRC-1 Archive
    if 'GetTransactionsResponse' in candid and 'ArchivedTransaction' in candid:
        return "ICRC-1 Archive", "infrastructure"
    if candid.count('BlockIndex') > 0 and candid.count('Transaction') > 0 and 'service : {}' in candid:
        return "ICRC-1 Archive", "infrastructure"

    # EVM RPC
    if 'EthMainnetService' in candid or 'EthSepoliaService' in candid:
        return "EVM RPC", "infrastructure"

    # Bitcoin Runes Etching
    if 'EtchingArgs' in candid and 'rune_name' in candid:
        return "Bitcoin Runes Etching", "infrastructure"

    return None, "candid_unrecognizable"

def set_project(cid, project):
    try:
        subprocess.run(['dfx', 'canister', '--network', 'ic', 'call', BACKEND, 'set_project',
                       f'(principal "{cid}", opt "{project}")'],
                      capture_output=True, timeout=60)
        return True
    except:
        return False

def research_canister(cid):
    print(f"Researching {cid}...", end=" ")

    # Try token
    name, symbol = query_icrc1(cid)
    if symbol:
        if symbol.endswith("•ODIN"):
            project = "ODIN.fun"
        elif symbol.startswith("ck"):
            project = f"{symbol} Ledger"
        else:
            project = f"{symbol} Token"
        print(f"✓ {project}")
        set_project(cid, project)
        return {"cid": cid, "project": project, "type": "token"}

    # Try index
    ledger_id = query_ledger_id(cid)
    if ledger_id:
        _, ledger_symbol = query_icrc1(ledger_id)
        if ledger_symbol:
            project = f"{ledger_symbol} Index"
            print(f"✓ {project}")
            set_project(cid, project)
            return {"cid": cid, "project": project, "type": "index"}

    # Check Candid
    candid = query_candid(cid)
    project, category = identify_by_candid(candid)
    if project:
        print(f"✓ {project}")
        set_project(cid, project)
        return {"cid": cid, "project": project, "category": category}

    # Unknown
    print(f"❌ Unknown ({category})")
    return {"cid": cid, "project": None, "reason": category, "candid": candid[:200] if candid else None}

# Process batch
if __name__ == '__main__':
    batch_start = 20  # Already researched 20
    batch_size = 30

    with open('/tmp/current_unknowns.json', 'r') as f:
        all_unknowns = json.load(f)

    batch = all_unknowns[batch_start:batch_start+batch_size]

    print(f"\\nProcessing batch {batch_start+1}-{batch_start+batch_size} of {len(all_unknowns)}\\n")

    results = []
    for i, cid in enumerate(batch, batch_start+1):
        print(f"[{i}/{len(all_unknowns)}] ", end="")
        result = research_canister(cid)
        results.append(result)
        time.sleep(0.5)

    # Save
    with open('/tmp/smart_batch_results.json', 'w') as f:
        json.dump(results, f, indent=2)

    identified = sum(1 for r in results if r.get('project'))
    print(f"\\n✓ Batch complete: {identified}/{len(results)} identified")
