#!/bin/bash
# CycleScan Deployment Script - Mainnet Only
# Usage: ./scripts/deploy.sh [--build-only|--deploy-only|--snapshot] [--test]

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_DIR="$( cd "$SCRIPT_DIR/.." && pwd )"

DEPLOY_TARGET="all"
RUN_TESTS=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --build-only)
            DEPLOY_TARGET="build"
            shift
            ;;
        --deploy-only)
            DEPLOY_TARGET="deploy"
            shift
            ;;
        --snapshot)
            DEPLOY_TARGET="snapshot"
            shift
            ;;
        --test)
            RUN_TESTS=true
            shift
            ;;
        --help)
            echo "CycleScan Deployment Script - Mainnet Only"
            echo ""
            echo "Usage: ./scripts/deploy.sh [options]"
            echo ""
            echo "Options:"
            echo "  --build-only    Only build the WASM, don't deploy"
            echo "  --deploy-only   Deploy without rebuilding"
            echo "  --snapshot      Take a snapshot of all tracked canisters"
            echo "  --test          Run post-deployment tests"
            echo "  --help          Show this help message"
            echo ""
            echo "Examples:"
            echo "  ./scripts/deploy.sh              # Build and deploy to mainnet"
            echo "  ./scripts/deploy.sh --snapshot   # Take a cycle snapshot"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo "================================================"
echo "CycleScan Deployment - MAINNET"
echo "================================================"
echo "Target: $DEPLOY_TARGET"
echo ""

cd "$PROJECT_DIR"

check_dfx() {
    if ! command -v dfx &> /dev/null; then
        echo "ERROR: dfx not installed"
        exit 1
    fi
}

use_daopad_identity() {
    export DFX_WARNING=-mainnet_plaintext_identity
    dfx identity use daopad
    echo "Identity: daopad"
    echo ""
}

build_backend() {
    echo "Building backend..."
    cargo build --release --target wasm32-unknown-unknown --package cyclescan_backend

    WASM_PATH="target/wasm32-unknown-unknown/release/cyclescan_backend.wasm"
    if [ -f "$WASM_PATH" ]; then
        WASM_SIZE=$(wc -c < "$WASM_PATH")
        echo "Backend WASM: $WASM_SIZE bytes"
    fi
    echo ""
}

build_frontend() {
    echo "Building frontend..."
    cd "$PROJECT_DIR/src/cyclescan_frontend"
    npx vite build 2>&1 | tail -5
    cd "$PROJECT_DIR"
    echo ""
}

deploy_backend() {
    echo "Deploying backend to mainnet..."
    dfx deploy cyclescan_backend --network ic --yes

    CANISTER_ID=$(dfx canister id cyclescan_backend --network ic)
    echo ""
    echo "Backend: $CANISTER_ID"
    echo "Dashboard: https://dashboard.internetcomputer.org/canister/$CANISTER_ID"
    echo ""
}

deploy_frontend() {
    echo "Deploying frontend to mainnet..."
    dfx deploy cyclescan_frontend --network ic --upgrade-unchanged

    CANISTER_ID=$(dfx canister id cyclescan_frontend --network ic)
    echo ""
    echo "Frontend: https://$CANISTER_ID.icp0.io/"
    echo ""
}

take_snapshot() {
    echo "Taking snapshot..."
    dfx canister call cyclescan_backend take_snapshot --network ic
    echo ""
}

run_tests() {
    echo "Testing..."
    echo "Stats:"
    dfx canister call cyclescan_backend get_stats --network ic
    echo ""
}

main() {
    check_dfx
    use_daopad_identity

    case $DEPLOY_TARGET in
        build)
            build_backend
            build_frontend
            ;;
        deploy)
            deploy_backend
            deploy_frontend
            ;;
        snapshot)
            take_snapshot
            ;;
        all)
            build_backend
            build_frontend
            deploy_backend
            deploy_frontend
            ;;
    esac

    if [ "$RUN_TESTS" = true ]; then
        run_tests
    fi

    BACKEND_ID=$(dfx canister id cyclescan_backend --network ic 2>/dev/null || echo "not deployed")
    FRONTEND_ID=$(dfx canister id cyclescan_frontend --network ic 2>/dev/null || echo "not deployed")
    echo "================================================"
    echo "Done!"
    echo "Backend:  $BACKEND_ID"
    echo "Frontend: https://$FRONTEND_ID.icp0.io/"
    echo "================================================"
}

main
