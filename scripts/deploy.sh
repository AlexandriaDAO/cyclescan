#!/bin/bash
# CycleScan Deployment Script - Frontend Only
# Usage: ./scripts/deploy.sh

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_DIR="$( cd "$SCRIPT_DIR/.." && pwd )"

echo "================================================"
echo "CycleScan Deployment - MAINNET"
echo "================================================"

cd "$PROJECT_DIR"

# Check dfx
if ! command -v dfx &> /dev/null; then
    echo "ERROR: dfx not installed"
    exit 1
fi

# Use daopad identity
export DFX_WARNING=-mainnet_plaintext_identity
dfx identity use daopad
echo "Identity: daopad"
echo ""

# Build frontend
echo "Building frontend..."
cd "$PROJECT_DIR/src/cyclescan_frontend"
npm run build 2>&1 | tail -5
cd "$PROJECT_DIR"
echo ""

# Deploy frontend
echo "Deploying frontend to mainnet..."
dfx deploy cyclescan_frontend --network ic --yes

CANISTER_ID=$(dfx canister id cyclescan_frontend --network ic)

echo "================================================"
echo "Done!"
echo "Frontend: https://$CANISTER_ID.icp0.io/"
echo "================================================"
