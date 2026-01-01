#!/bin/bash
# Website Corrections Script for CycleScan
# Generated: 2026-01-01
# This script corrects known website errors in the backend

set -e

CANISTER_ID="vohji-riaaa-aaaac-babxq-cai"

echo "Applying website corrections to CycleScan backend..."
echo ""

# Fix ICPanda (from raw icp0.io URL to panda.fans)
echo "Fixing ICPanda..."
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "d7wvo-iiaaa-aaaaq-aacsq-cai", record { website = opt opt "https://panda.fans" })'
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "c3324-riaaa-aaaaq-aacuq-cai", record { website = opt opt "https://panda.fans" })'

# Fix Neutrinite (from boomdao.xyz to icpcoins.com)
echo "Fixing Neutrinite..."
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "togwv-zqaaa-aaaal-qr7aa-cai", record { website = opt opt "https://icpcoins.com" })'
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "6jvpj-sqaaa-aaaaj-azwnq-cai", record { website = opt opt "https://icpcoins.com" })'
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "ft6fn-7aaaa-aaaaq-aacfa-cai", record { website = opt opt "https://icpcoins.com" })'
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "nzsmr-6iaaa-aaaal-qsnea-cai", record { website = opt opt "https://icpcoins.com" })'
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "7ew52-sqaaa-aaaal-qsrda-cai", record { website = opt opt "https://icpcoins.com" })'
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "f54if-eqaaa-aaaaq-aacea-cai", record { website = opt opt "https://icpcoins.com" })'

# Fix BOOM (from icp0.io URL to boomdao.xyz)
echo "Fixing BOOM..."
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "v5tde-5aaaa-aaaaq-aabja-cai", record { website = opt opt "https://boomdao.xyz" })'
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "xjngq-yaaaa-aaaaq-aabha-cai", record { website = opt opt "https://boomdao.xyz" })'

# Fix ICVC (from vercel URL to ic-vc.com)
echo "Fixing ICVC..."
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "mqvz3-xaaaa-aaaaq-aadva-cai", record { website = opt opt "https://ic-vc.com" })'
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "nuywj-oaaaa-aaaaq-aadta-cai", record { website = opt opt "https://ic-vc.com" })'

# Fix GOLDAO (from docs subdomain to main domain)
echo "Fixing GOLDAO..."
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "tr3th-kiaaa-aaaaq-aab6q-cai", record { website = opt opt "https://www.gold-dao.org" })'
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "efv5g-kqaaa-aaaaq-aacaa-cai", record { website = opt opt "https://www.gold-dao.org" })'
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "tw2vt-hqaaa-aaaaq-aab6a-cai", record { website = opt opt "https://www.gold-dao.org" })'

# Fix TACO (remove incorrect internetcomputer.org - set to null)
echo "Fixing TACO..."
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "lacdn-3iaaa-aaaaq-aae3a-cai", record { website = opt null })'
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "kepm7-ciaaa-aaaaq-aae5a-cai", record { website = opt null })'
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "v6t5d-6yaaa-aaaan-qzzja-cai", record { website = opt null })'
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "kknbx-zyaaa-aaaaq-aae4a-cai", record { website = opt null })'

# Fix Draggin Karma Points (from internetcomputer.org to dragginz.io)
echo "Fixing Draggin Karma Points..."
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "zlaol-iaaaa-aaaaq-aaaha-cai", record { website = opt opt "https://dragginz.io" })'
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "zfcdd-tqaaa-aaaaq-aaaga-cai", record { website = opt opt "https://dragginz.io" })'

# Fix Mimic Clay (remove incorrect internetcomputer.org)
echo "Fixing Mimic Clay..."
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "5ithz-aqaaa-aaaaq-aaa4a-cai", record { website = opt null })'
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "4c4fd-caaaa-aaaaq-aaa3a-cai", record { website = opt null })'

# Fix DecideAI (from internetcomputer.org to decideai.xyz)
echo "Fixing DecideAI..."
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "xaonm-oiaaa-aaaaq-aabgq-cai", record { website = opt opt "https://decideai.xyz" })'

# Fix KINIC (from internetcomputer.org to kinic.io)
echo "Fixing KINIC..."
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "7vojr-tyaaa-aaaaq-aaatq-cai", record { website = opt opt "https://kinic.io" })'
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "7jkta-eyaaa-aaaaq-aaarq-cai", record { website = opt opt "https://kinic.io" })'

# Fix ALICE (from internetcomputer.org to alice.fun)
echo "Fixing ALICE..."
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "mtcaz-pyaaa-aaaaq-aaeia-cai", record { website = opt opt "https://alice.fun" })'

# Fix TRAX (from internetcomputer.org to trax.so)
echo "Fixing TRAX..."
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "e6qbd-qiaaa-aaaaq-aaccq-cai", record { website = opt opt "https://trax.so" })'

# Fix ESTATE (from internetcomputer.org to estatedao.org)
echo "Fixing ESTATE..."
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "bfk5s-wyaaa-aaaaq-aac5q-cai", record { website = opt opt "https://estatedao.org" })'

# Fix PHASMA (remove incorrect internetcomputer.org)
echo "Fixing PHASMA..."
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "n535v-yiaaa-aaaaq-aadsq-cai", record { website = opt null })'

# Fix FomoWell (remove incorrect internetcomputer.org)
echo "Fixing FomoWell..."
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "os3ua-lqaaa-aaaaq-aaefq-cai", record { website = opt null })'
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "pww3s-sqaaa-aaaaq-aaedq-cai", record { website = opt null })'

# Fix Swampies (from internetcomputer.org to dragginz.io)
echo "Fixing Swampies..."
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "ldv2p-dqaaa-aaaaq-aadga-cai", record { website = opt opt "https://dragginz.io" })'
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "l7ra6-uqaaa-aaaaq-aadea-cai", record { website = opt opt "https://dragginz.io" })'

# Fix Nuance (from home.nuance.xyz to nuance.xyz)
echo "Fixing Nuance..."
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "q5mdq-biaaa-aaaaq-aabuq-cai", record { website = opt opt "https://nuance.xyz" })'
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "rzbmc-yiaaa-aaaaq-aabsq-cai", record { website = opt opt "https://nuance.xyz" })'

# Fix ICPEx (from internetcomputer.org to icpex.org)
echo "Fixing ICPEx..."
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "l3h7i-bqaaa-aaaaq-aaezq-cai", record { website = opt opt "https://icpex.org" })'
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "2vf3u-cqaaa-aaaam-ab5ha-cai", record { website = opt opt "https://icpex.org" })'
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "jpz24-eqaaa-aaaaq-aaexq-cai", record { website = opt opt "https://icpex.org" })'

# Fix DOLR AI (remove incorrect internetcomputer.org)
echo "Fixing DOLR AI..."
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "6dfr2-giaaa-aaaaq-aaawq-cai", record { website = opt null })'
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "67bll-riaaa-aaaaq-aaauq-cai", record { website = opt null })'

# Add missing websites
echo "Adding missing websites..."

# Add BoB (Burn or Burn) website
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "6lnhz-oaaaa-aaaas-aabkq-cai", record { website = opt opt "https://bob.fun" })'

# Add Omnity website
dfx canister --network ic call $CANISTER_ID update_canister \
  '(principal "pw3ee-pyaaa-aaaar-qahva-cai", record { website = opt opt "https://www.omnity.network" })'

echo ""
echo "Website corrections applied successfully!"
echo "Run 'dfx canister --network ic call $CANISTER_ID get_project_leaderboard' to verify."
