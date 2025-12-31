#!/bin/bash
set -e
cd /home/theseus/alexandria/cyclescan
export DFX_WARNING=-mainnet_plaintext_identity
dfx canister call cyclescan_backend take_snapshot --network ic
