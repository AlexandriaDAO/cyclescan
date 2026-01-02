#!/usr/bin/env python3
"""Fix generic DEX and NFT project assignments."""
import subprocess

CANISTER_ID = "vohji-riaaa-aaaac-babxq-cai"

def run_dfx(method, candid):
    result = subprocess.run(
        ['dfx', 'canister', 'call', CANISTER_ID, method, candid, '--network', 'ic'],
        capture_output=True, text=True
    )
    if result.returncode != 0:
        print(f"Error: {result.stderr}")
        return None
    return result.stdout.strip()

# Create RunicSwap project
print("Creating RunicSwap project...")
run_dfx('import_projects', '(vec { record { name = "RunicSwap"; website = opt "https://github.com/buriburizaemonnn/RunicSwap" } })')

# Reassign DEX canister to RunicSwap
print("Reassigning DEX canister to RunicSwap...")
candid = '''(vec { record {
    canister_id = principal "h43eb-lqaaa-aaaao-qjxgq-cai";
    proxy_id = principal "e3mmv-5qaaa-aaaah-aadma-cai";
    proxy_type = variant { Blackhole };
    project = opt "RunicSwap";
    valid = opt true
} })'''
result = run_dfx('import_canisters', candid)
print(f"Result: {result}")

# Reassign second NFT canister to ESTATE
print("Reassigning uninitialized NFT canister to ESTATE...")
candid = '''(vec { record {
    canister_id = principal "ekbv3-lqaaa-aaaap-ab4oq-cai";
    proxy_id = principal "e3mmv-5qaaa-aaaah-aadma-cai";
    proxy_type = variant { Blackhole };
    project = opt "ESTATE";
    valid = opt true
} })'''
result = run_dfx('import_canisters', candid)
print(f"Result: {result}")

print("\nDone! The generic projects are now properly identified.")
