#!/bin/bash
# Research a single canister following the playbook
# Usage: ./research_canister.sh <canister_id>

set -e

CANISTER_ID="$1"
BACKEND="vohji-riaaa-aaaac-babxq-cai"

if [ -z "$CANISTER_ID" ]; then
    echo "Usage: $0 <canister_id>"
    exit 1
fi

echo "=========================================="
echo "Researching: $CANISTER_ID"
echo "=========================================="

# Step 1: Query Candid metadata
echo -e "\n[Step 1] Querying Candid metadata..."
CANDID_OUTPUT=$(dfx canister --network ic metadata "$CANISTER_ID" candid:service 2>&1 | head -100 || true)

if echo "$CANDID_OUTPUT" | grep -q "Failed to read"; then
    echo "  ❌ No Candid metadata available"
    CANDID_AVAILABLE="false"
else
    echo "  ✓ Candid available"
    CANDID_AVAILABLE="true"
    echo "$CANDID_OUTPUT" | head -20
fi

# Step 2: Try ICRC-1 token methods
echo -e "\n[Step 2] Trying ICRC-1 token methods..."
TOKEN_NAME=$(dfx canister --network ic call "$CANISTER_ID" icrc1_name '()' 2>&1 || true)
TOKEN_SYMBOL=$(dfx canister --network ic call "$CANISTER_ID" icrc1_symbol '()' 2>&1 || true)

if echo "$TOKEN_NAME" | grep -q '("'; then
    TOKEN_NAME_CLEAN=$(echo "$TOKEN_NAME" | grep -oP '\("\K[^"]+' || echo "")
    TOKEN_SYMBOL_CLEAN=$(echo "$TOKEN_SYMBOL" | grep -oP '\("\K[^"]+' || echo "")
    echo "  ✓ ICRC-1 Token detected!"
    echo "    Name: $TOKEN_NAME_CLEAN"
    echo "    Symbol: $TOKEN_SYMBOL_CLEAN"

    # Check for ODIN pattern
    if echo "$TOKEN_SYMBOL_CLEAN" | grep -q "•ODIN$"; then
        PROJECT="ODIN.fun"
        echo "  ✓ Identified as ODIN.fun token"
        echo "{\"canister_id\": \"$CANISTER_ID\", \"project\": \"$PROJECT\", \"token_name\": \"$TOKEN_NAME_CLEAN\", \"token_symbol\": \"$TOKEN_SYMBOL_CLEAN\", \"category\": \"token\"}"
        exit 0
    fi

    # Check for ck* pattern
    if echo "$TOKEN_SYMBOL_CLEAN" | grep -q "^ck"; then
        PROJECT="$TOKEN_SYMBOL_CLEAN Ledger"
        echo "  ✓ Identified as $PROJECT"
        echo "{\"canister_id\": \"$CANISTER_ID\", \"project\": \"$PROJECT\", \"token_name\": \"$TOKEN_NAME_CLEAN\", \"token_symbol\": \"$TOKEN_SYMBOL_CLEAN\", \"category\": \"nns_infrastructure\"}"
        exit 0
    fi

    echo "  ⚠ Token found but project unknown: $TOKEN_NAME_CLEAN ($TOKEN_SYMBOL_CLEAN)"
    echo "{\"canister_id\": \"$CANISTER_ID\", \"token_name\": \"$TOKEN_NAME_CLEAN\", \"token_symbol\": \"$TOKEN_SYMBOL_CLEAN\", \"project\": null, \"reason\": \"token_unknown_project\"}"
    exit 0
else
    echo "  ❌ Not an ICRC-1 token"
fi

# Step 3: Try index canister methods
echo -e "\n[Step 3] Trying index canister methods..."
LEDGER_ID=$(dfx canister --network ic call "$CANISTER_ID" ledger_id '()' 2>&1 || true)

if echo "$LEDGER_ID" | grep -q 'principal'; then
    LEDGER_ID_CLEAN=$(echo "$LEDGER_ID" | grep -oP 'principal "\K[^"]+' || echo "")
    echo "  ✓ Index canister detected!"
    echo "    Ledger ID: $LEDGER_ID_CLEAN"

    # Query ledger for token info
    LEDGER_NAME=$(dfx canister --network ic call "$LEDGER_ID_CLEAN" icrc1_name '()' 2>&1 || true)
    LEDGER_SYMBOL=$(dfx canister --network ic call "$LEDGER_ID_CLEAN" icrc1_symbol '()' 2>&1 || true)

    if echo "$LEDGER_SYMBOL" | grep -q '("'; then
        LEDGER_SYMBOL_CLEAN=$(echo "$LEDGER_SYMBOL" | grep -oP '\("\K[^"]+' || echo "")
        PROJECT="$LEDGER_SYMBOL_CLEAN Index"
        echo "  ✓ Identified as $PROJECT"
        echo "{\"canister_id\": \"$CANISTER_ID\", \"project\": \"$PROJECT\", \"ledger_id\": \"$LEDGER_ID_CLEAN\", \"category\": \"nns_infrastructure\"}"
        exit 0
    fi
else
    echo "  ❌ Not an index canister"
fi

# Step 4: Output what we found
echo -e "\n[Step 4] Summary"
echo "  Candid available: $CANDID_AVAILABLE"
echo "  Unable to auto-identify project"
echo "{\"canister_id\": \"$CANISTER_ID\", \"candid_available\": $CANDID_AVAILABLE, \"project\": null, \"reason\": \"needs_manual_research\"}"
