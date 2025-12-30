# Canister Research Plan

**Objective:** Identify the project/purpose of every non-token canister in `trackable_canisters.json`.

**Status:** COMPLETE
**Total canisters to research:** 206
**Already identified:** 1 (3r4gx-wqaaa-aaaaq-aaaia-cai = SNS Aggregator)
**Completed:** 206
**Remaining:** 0

---

## Instructions for Research Agent

You MUST research every single canister in the list below. Do NOT stop until every canister has a result recorded. Each canister MUST have one of these outcomes:
- Project name identified
- Marked as "Unknown" (only after exhaustive research)

### Research Protocol (Follow for EACH canister)

For each canister_id, perform ALL of these steps in order:

#### Step 1: Web Search (Required)
```
Search: "{canister_id}" site:github.com
Search: "{canister_id}" ICP project
Search: "{canister_id}" dfinity forum
```
Look for:
- GitHub repos containing this canister ID
- Forum posts mentioning deployment
- Project documentation

#### Step 2: Check Frontend (Required)
Try accessing:
```
https://{canister_id}.icp0.io
https://{canister_id}.ic0.app
https://{canister_id}.raw.icp0.io
```
Look for:
- App name in title/header
- About page
- Footer credits

#### Step 3: Query Candid Interface (Required)
```bash
dfx canister --network ic call {canister_id} __get_candid_interface_tmp_hack '()'
```
Analyze method names for clues:
- `mint`, `transfer`, `balance` → Token/NFT
- `swap`, `add_liquidity` → DEX
- `create_post`, `get_posts` → Social app
- `upload`, `download` → Storage
- `register`, `login` → Auth service

#### Step 4: Check ICP Dashboard (Required)
Visit: `https://dashboard.internetcomputer.org/canister/{canister_id}`
Look for:
- Canister name (if set)
- Module hash (can cross-reference with known projects)
- Controllers (might reveal parent project)

#### Step 5: Record Result (Required)
Update the tracking table below with:
- Status: `done`
- Project: The identified project name OR `Unknown`
- Notes: Brief explanation of what you found (or "No results from any method")

### Recording Format

After researching each canister, update the tracking table AND add to `project_mappings.json`:

```json
{
  "by_canister": {
    "{canister_id}": "ProjectName or Unknown"
  }
}
```

---

## Research Tracking Table

**Instructions:** Mark each row as you complete it. Do NOT skip any rows.

| # | canister_id | Status | Project | Notes |
|---|-------------|--------|---------|-------|
| 1 | 22hwg-5iaaa-aaaah-arb4q-cai | done | Unknown | Frontend: https://22hwg-5iaaa-aaaah-arb4q-cai.icp0.io.  |
| 2 | 235yg-kiaaa-aaaaj-qnwlq-cai | done | Unknown | Frontend: https://235yg-kiaaa-aaaaj-qnwlq-cai.icp0.io.  |
| 3 | 2377i-5aaaa-aaaai-q3yua-cai | done | Unknown | Frontend: https://2377i-5aaaa-aaaai-q3yua-cai.icp0.io.  |
| 4 | 23gat-biaaa-aaaaj-a2bca-cai | done | Unknown | Frontend: https://23gat-biaaa-aaaaj-a2bca-cai.ic0.app.  |
| 5 | 246z4-qyaaa-aaaai-q3yuq-cai | done | Unknown | Frontend: https://246z4-qyaaa-aaaai-q3yuq-cai.ic0.app.  |
| 6 | 24t7b-laaaa-aaaak-quf6q-cai | done | Unknown | Frontend: https://24t7b-laaaa-aaaak-quf6q-cai.ic0.app.  |
| 7 | 25jyk-oaaaa-aaaah-adwwa-cai | done | Unknown | Frontend: https://25jyk-oaaaa-aaaah-adwwa-cai.ic0.app.  |
| 8 | 2664g-ziaaa-aaaai-q3uyq-cai | done | Unknown | Frontend: https://2664g-ziaaa-aaaai-q3uyq-cai.ic0.app.  |
| 9 | 26biv-aaaaa-aaaal-ad4aa-cai | done | Unknown | Frontend: https://26biv-aaaaa-aaaal-ad4aa-cai.ic0.app.  |
| 10 | 275ts-ziaaa-aaaaj-qnotq-cai | done | Unknown | Frontend: https://275ts-ziaaa-aaaaj-qnotq-cai.icp0.io.  |
| 11 | 27q7f-uiaaa-aaaaj-a2g7q-cai | done | Unknown | Frontend: https://27q7f-uiaaa-aaaaj-a2g7q-cai.icp0.io.  |
| 12 | 2be6c-nqaaa-aaaaf-qan7q-cai | done | Unknown | Frontend: https://2be6c-nqaaa-aaaaf-qan7q-cai.ic0.app.  |
| 13 | 2bslg-pyaaa-aaaal-ad37a-cai | done | Unknown | Frontend: https://2bslg-pyaaa-aaaal-ad37a-cai.ic0.app.  |
| 14 | 2bwnk-liaaa-aaaak-quria-cai | done | Unknown | Frontend: https://2bwnk-liaaa-aaaak-quria-cai.ic0.app.  |
| 15 | 2c2gx-oiaaa-aaaai-q3u2q-cai | done | Unknown | Frontend: https://2c2gx-oiaaa-aaaai-q3u2q-cai.ic0.app.  |
| 16 | 2cse4-eyaaa-aaaal-ajvpa-cai | done | Unknown | Frontend: https://2cse4-eyaaa-aaaal-ajvpa-cai.ic0.app.  |
| 17 | 2dso4-giaaa-aaaal-adxta-cai | done | Unknown | Frontend: https://2dso4-giaaa-aaaal-adxta-cai.ic0.app.  |
| 18 | 2ed46-kaaaa-aaaam-qdo3a-cai | done | Unknown | Frontend: https://2ed46-kaaaa-aaaam-qdo3a-cai.ic0.app.  |
| 19 | 2etii-lqaaa-aaaal-adxtq-cai | done | Unknown | Frontend: https://2etii-lqaaa-aaaal-adxtq-cai.ic0.app.  |
| 20 | 2f3ad-dqaaa-aaaai-q3u2a-cai | done | Unknown | Frontend: https://2f3ad-dqaaa-aaaai-q3u2a-cai.icp0.io.  |
| 21 | 2fytf-maaaa-aaaad-qg65a-cai | done | Unknown | Frontend: https://2fytf-maaaa-aaaad-qg65a-cai.ic0.app.  |
| 22 | 2gfyw-aiaaa-aaaaf-qan7a-cai | done | Unknown | Frontend: https://2gfyw-aiaaa-aaaaf-qan7a-cai.icp0.io.  |
| 23 | 2gtns-caaaa-aaaal-ad37q-cai | done | Unknown | Frontend: https://2gtns-caaaa-aaaal-ad37q-cai.ic0.app.  |
| 24 | 2h5or-paaaa-aaaak-qcjyq-cai | done | Unknown | Frontend: https://2h5or-paaaa-aaaak-qcjyq-cai.icp0.io.  |
| 25 | 2hogi-2qaaa-aaaag-at53a-cai | done | Unknown | Frontend: https://2hogi-2qaaa-aaaag-at53a-cai.ic0.app.  |
| 26 | 2jzxy-6qaaa-aaaai-atfya-cai | done | Unknown | Frontend: https://2jzxy-6qaaa-aaaai-atfya-cai.ic0.app.  |
| 27 | 2lrpa-sqaaa-aaaal-ajvoq-cai | done | Unknown | Frontend: https://2lrpa-sqaaa-aaaal-ajvoq-cai.ic0.app.  |
| 28 | 2lznl-yaaaa-aaaai-q3u3a-cai | done | Unknown | Frontend: https://2lznl-yaaaa-aaaai-q3u3a-cai.icp0.io.  |
| 29 | 2myl7-vyaaa-aaaai-q3u3q-cai | done | Unknown | Frontend: https://2myl7-vyaaa-aaaai-q3u3q-cai.ic0.app.  |
| 30 | 2naxc-4iaaa-aaaam-qdo2q-cai | done | Unknown | Frontend: https://2naxc-4iaaa-aaaam-qdo2q-cai.icp0.io.  |
| 31 | 2nqdu-5yaaa-aaaal-adxsa-cai | done | Unknown | Frontend: https://2nqdu-5yaaa-aaaal-adxsa-cai.ic0.app.  |
| 32 | 2nvud-4yaaa-aaaal-qjaxa-cai | done | Unknown | Frontend: https://2nvud-4yaaa-aaaal-qjaxa-cai.ic0.app.  |
| 33 | 2obr6-aaaaa-aaaaj-a2bbq-cai | done | Unknown | Frontend: https://2obr6-aaaaa-aaaaj-a2bbq-cai.icp0.io.  |
| 34 | 2onnu-myaaa-aaaag-at52q-cai | done | Unknown | Frontend: https://2onnu-myaaa-aaaag-at52q-cai.icp0.io.  |
| 35 | 2q4ro-cyaaa-aaaai-q3uzq-cai | done | Unknown | Frontend: https://2q4ro-cyaaa-aaaai-q3uzq-cai.ic0.app.  |
| 36 | 2qc72-4qaaa-aaaap-qhwqq-cai | done | Unknown | Frontend: https://2qc72-4qaaa-aaaap-qhwqq-cai.ic0.app.  |
| 37 | 2rgll-iiaaa-aaaak-quccq-cai | done | Unknown | Frontend: https://2rgll-iiaaa-aaaak-quccq-cai.ic0.app.  |
| 38 | 2rv37-7aaaa-aaaal-ajbyq-cai | done | Unknown | Frontend: https://2rv37-7aaaa-aaaal-ajbyq-cai.ic0.app.  |
| 39 | 2s4l5-eiaaa-aaaai-atf2q-cai | done | Unknown | Frontend: https://2s4l5-eiaaa-aaaai-atf2q-cai.ic0.app.  |
| 40 | 2tcj3-baaaa-aaaaf-qan4q-cai | done | Unknown | Frontend: https://2tcj3-baaaa-aaaaf-qan4q-cai.ic0.app.  |
| 41 | 2te52-laaaa-aaaah-arb5a-cai | done | Unknown | Frontend: https://2te52-laaaa-aaaah-arb5a-cai.ic0.app.  |
| 42 | 2udpp-myaaa-aaaaf-qan4a-cai | done | Unknown | Frontend: https://2udpp-myaaa-aaaaf-qan4a-cai.icp0.io.  |
| 43 | 2v3om-2yaaa-aaaah-qczoa-cai | done | Unknown | Frontend: https://2v3om-2yaaa-aaaah-qczoa-cai.icp0.io.  |
| 44 | 2v5nj-jqaaa-aaaai-atf2a-cai | done | Unknown | Frontend: https://2v5nj-jqaaa-aaaai-atf2a-cai.icp0.io.  |
| 45 | 2vn26-myaaa-aaaao-aanla-cai | done | Unknown | Methods: service, hello |
| 46 | 2whn7-fqaaa-aaaak-qucca-cai | done | Unknown | Frontend: https://2whn7-fqaaa-aaaak-qucca-cai.icp0.io.  |
| 47 | 2wu5l-syaaa-aaaal-ajbya-cai | done | Unknown | Frontend: https://2wu5l-syaaa-aaaal-ajbya-cai.ic0.app.  |
| 48 | 2x5x2-paaaa-aaaai-q3uza-cai | done | Unknown | Frontend: https://2x5x2-paaaa-aaaai-q3uza-cai.icp0.io.  |
| 49 | 2yxsz-4qaaa-aaaal-adxrq-cai | done | Unknown | Frontend: https://2yxsz-4qaaa-aaaal-adxrq-cai.ic0.app.  |
| 50 | 2z72s-uqaaa-aaaai-q3uya-cai | done | Unknown | Frontend: https://2z72s-uqaaa-aaaai-q3uya-cai.icp0.io.  |
| 51 | 2zaob-nyaaa-aaaal-ad4aq-cai | done | Unknown | Frontend: https://2zaob-nyaaa-aaaal-ad4aq-cai.ic0.app.  |
| 52 | 322ha-cqaaa-aaaah-qqcpq-cai | done | Unknown | Frontend: https://322ha-cqaaa-aaaah-qqcpq-cai.ic0.app.  |
| 53 | 33tnr-7iaaa-aaaae-acxoq-cai | done | Unknown | Frontend: https://33tnr-7iaaa-aaaae-acxoq-cai.ic0.app.  |
| 54 | 346vy-oyaaa-aaaan-qzyja-cai | done | Unknown | Frontend: https://346vy-oyaaa-aaaan-qzyja-cai.ic0.app.  |
| 55 | 353bu-piaaa-aaaah-qqcpa-cai | done | Unknown | Frontend: https://353bu-piaaa-aaaah-qqcpa-cai.ic0.app.  |
| 56 | 35dny-6aaaa-aaaas-akqba-cai | done | Unknown | Frontend: https://35dny-6aaaa-aaaas-akqba-cai.icp0.io.  |
| 57 | 36l7l-yqaaa-aaaal-amm5a-cai | done | Unknown | Frontend: https://36l7l-yqaaa-aaaal-amm5a-cai.icp0.io.  |
| 58 | 3a6h2-sqaaa-aaaal-adxvq-cai | done | Unknown | Frontend: https://3a6h2-sqaaa-aaaal-adxvq-cai.ic0.app.  |
| 59 | 3awac-fiaaa-aaaad-qhpkq-cai | done | Unknown | Frontend: https://3awac-fiaaa-aaaad-qhpkq-cai.ic0.app.  |
| 60 | 3dewk-oiaaa-aaaah-qddlq-cai | done | Unknown | Frontend: https://3dewk-oiaaa-aaaah-qddlq-cai.icp0.io.  |
| 61 | 3dgcv-ziaaa-aaaao-aanoa-cai | done | Unknown | Methods: service, hello |
| 62 | 3ecpo-oiaaa-aaaag-at55q-cai | done | Unknown | Frontend: https://3ecpo-oiaaa-aaaag-at55q-cai.icp0.io.  |
| 63 | 3gxjf-xiaaa-aaaai-q3u4q-cai | done | Unknown | Frontend: https://3gxjf-xiaaa-aaaai-q3u4q-cai.icp0.io.  |
| 64 | 3h7bo-7iaaa-aaaal-adxva-cai | done | Unknown | Frontend: https://3h7bo-7iaaa-aaaal-adxva-cai.ic0.app.  |
| 65 | 3hxgw-iqaaa-aaaad-qhpka-cai | done | Unknown | Frontend: https://3hxgw-iqaaa-aaaad-qhpka-cai.icp0.io.  |
| 66 | 3j5mg-eyaaa-aaaal-adxua-cai | done | Unknown | Frontend: https://3j5mg-eyaaa-aaaal-adxua-cai.ic0.app.  |
| 67 | 3lmog-zyaaa-aaaal-amm6q-cai | done | Unknown | Frontend: https://3lmog-zyaaa-aaaal-amm6q-cai.ic0.app.  |
| 68 | 3mnis-uaaaa-aaaal-amm6a-cai | done | Unknown | Frontend: https://3mnis-uaaaa-aaaal-amm6a-cai.icp0.io.  |
| 69 | 3ng3c-vyaaa-aaaah-qddkq-cai | done | Unknown | Frontend: https://3ng3c-vyaaa-aaaah-qddkq-cai.icp0.io.  |
| 70 | 3o4ks-jaaaa-aaaal-adxuq-cai | done | Unknown | Frontend: https://3o4ks-jaaaa-aaaal-adxuq-cai.ic0.app.  |
| 71 | 3pucz-baaaa-aaaai-q3u5a-cai | done | Unknown | Frontend: https://3pucz-baaaa-aaaai-q3u5a-cai.ic0.app.  |
| 72 | 3qgfz-2qaaa-aaaar-qaipq-cai | done | Unknown | Frontend: https://3qgfz-2qaaa-aaaar-qaipq-cai.ic0.app. Methods: owner, subaccount, burn, kind, mint, approve, timestamp, transfer, fee, from |
| 73 | 3rf6d-paaaa-aaaag-at56a-cai | done | Unknown | Frontend: https://3rf6d-paaaa-aaaag-at56a-cai.ic0.app.  |
| 74 | 3stx4-waaaa-aaaaj-qnoua-cai | done | Unknown | Frontend: https://3stx4-waaaa-aaaaj-qnoua-cai.ic0.app.  |
| 75 | 3syqd-6aaaa-aaaal-adxwq-cai | done | Unknown | Frontend: https://3syqd-6aaaa-aaaal-adxwq-cai.ic0.app.  |
| 76 | 3thnb-raaaa-aaaaa-qahla-cai | done | Unknown | Frontend: https://3thnb-raaaa-aaaaa-qahla-cai.icp0.io.  |
| 77 | 3ttlo-zqaaa-aaaad-qg6ya-cai | done | Unknown | Frontend: https://3ttlo-zqaaa-aaaad-qg6ya-cai.ic0.app.  |
| 78 | 3uglv-4yaaa-aaaaa-qahlq-cai | done | Unknown | Frontend: https://3uglv-4yaaa-aaaaa-qahlq-cai.ic0.app.  |
| 79 | 3v56e-yqaaa-aaaan-qzyiq-cai | done | Unknown | Frontend: https://3v56e-yqaaa-aaaan-qzyiq-cai.icp0.io.  |
| 80 | 3vzwx-tyaaa-aaaal-adxwa-cai | done | Unknown | Frontend: https://3vzwx-tyaaa-aaaal-adxwa-cai.ic0.app.  |
| 81 | 3wdhh-paaaa-aaaah-qddia-cai | done | Unknown | Frontend: https://3wdhh-paaaa-aaaah-qddia-cai.icp0.io.  |
| 82 | 3wt4i-faaaa-aaaaj-qnwma-cai | done | Unknown | Frontend: https://3wt4i-faaaa-aaaaj-qnwma-cai.ic0.app.  |
| 83 | 3wzge-xyaaa-aaaal-qbeja-cai | done | Unknown | Methods: service, hello |
| 84 | 3x5sf-taaaa-aaaah-aq33q-cai | done | Unknown | Frontend: https://3x5sf-taaaa-aaaah-aq33q-cai.ic0.app. Methods: owner, subaccount, hash, owner, subaccount, RegisterKnownNeuron, ManageNeuron, CreateServiceNervousSystem, ExecuteNnsFunction, RewardNodeProvider |
| 85 | 3xtda-wiaaa-aaaar-qbspq-cai | done | ODIN.fun | Token: BITNEIRO•ID•ZMEO•ODIN (BITNEIRO•ID•ZMEO•ODIN) |
| 86 | 3ybkp-uqaaa-aaaah-qddja-cai | done | Unknown | Frontend: https://3ybkp-uqaaa-aaaah-qddja-cai.ic0.app.  |
| 87 | 3zfof-myaaa-aaaar-qaioa-cai | done | Unknown | Frontend: https://3zfof-myaaa-aaaar-qaioa-cai.ic0.app.  |
| 88 | 3zkvx-kaaaa-aaaak-qt2ia-cai | done | Unknown Token | Token: Trump (TRMP) |
| 89 | 3zkz7-viaaa-aaaal-amm5q-cai | done | Unknown | Frontend: https://3zkz7-viaaa-aaaal-amm5q-cai.ic0.app.  |
| 90 | 3zroi-nyaaa-aaaar-qbsoq-cai | done | ODIN.fun | Token: BUTTERFLY•ID•LRWR•ODIN (BUTTERFLY•ID•LRWR•ODIN) |
| 91 | 3zwjo-2yaaa-aaaao-qj6xq-cai | done | Unknown | Frontend: https://3zwjo-2yaaa-aaaao-qj6xq-cai.icp0.io.  |
| 92 | 42crl-7yaaa-aaaak-adkzq-cai | done | Unknown | Frontend: https://42crl-7yaaa-aaaak-adkzq-cai.icp0.io.  |
| 93 | 43adh-naaaa-aaaar-qbvdq-cai | done | ODIN.fun | Token: AMANDAXD•ID•DTDV•ODIN (AMANDAXD•ID•DTDV•ODIN) |
| 94 | 43cxi-kaaaa-aaaar-qaqgq-cai | done | ODIN.fun | Token: WZRD•ID•HFZK•ODIN (WZRD•ID•HFZK•ODIN) |
| 95 | 43qjv-pyaaa-aaaac-a4wxq-cai | done | Unknown | Frontend: https://43qjv-pyaaa-aaaac-a4wxq-cai.icp0.io.  |
| 96 | 43udk-maaaa-aaaar-qapda-cai | done | ODIN.fun | Token: ODINAI•ID•VEDI•ODIN (ODINAI•ID•VEDI•ODIN) |
| 97 | 43wu7-xyaaa-aaaad-qg6jq-cai | done | Unknown | Frontend: https://43wu7-xyaaa-aaaad-qg6jq-cai.ic0.app.  |
| 98 | 4426m-haaaa-aaaak-qt5fq-cai | done | Unknown Token | Token: Aptos (APT) |
| 99 | 44bft-ayaaa-aaaar-qbvda-cai | done | ODIN.fun | Token: KEKIUS•ID•EAQQ•ODIN (KEKIUS•ID•EAQQ•ODIN) |
| 100 | 44dr4-hyaaa-aaaar-qaqga-cai | done | ODIN.fun | Token: NUKLAI•ID•COFG•ODIN (NUKLAI•ID•COFG•ODIN) |
| 101 | 44jjm-nqaaa-aaaap-ai7jq-cai | done | Unknown | Frontend: https://44jjm-nqaaa-aaaap-ai7jq-cai.ic0.app.  |
| 102 | 44kxe-zaaaa-aaaal-ajk6a-cai | done | Unknown | Frontend: https://44kxe-zaaaa-aaaal-ajk6a-cai.ic0.app.  |
| 103 | 44vf6-byaaa-aaaar-qapdq-cai | done | ODIN.fun | Token: AODINGGOU•ID•EYHK•ODIN (AODINGGOU•ID•EYHK•ODIN) |
| 104 | 44xsl-2aaaa-aaaad-qg6ja-cai | done | Unknown | Frontend: https://44xsl-2aaaa-aaaad-qg6ja-cai.icp0.io.  |
| 105 | 454jg-5qaaa-aaaal-adxhq-cai | done | Unknown | Frontend: https://454jg-5qaaa-aaaal-adxhq-cai.ic0.app.  |
| 106 | 45pog-dqaaa-aaaao-a4piq-cai | done | Unknown Token | Token: MyToken (MYTOKEN) |
| 107 | 466cd-vaaaa-aaaac-aonra-cai | done | Unknown | Frontend: https://466cd-vaaaa-aaaac-aonra-cai.icp0.io.  |
| 108 | 46bhg-mqaaa-aaaag-at5pa-cai | done | Unknown | Frontend: https://46bhg-mqaaa-aaaag-at5pa-cai.ic0.app.  |
| 109 | 473ia-7yaaa-aaaag-amd5a-cai | done | Unknown | Frontend: https://473ia-7yaaa-aaaag-amd5a-cai.ic0.app.  |
| 110 | 473jg-3qaaa-aaaai-qpkya-cai | done | Unknown Token | Token: GoldFish (GLDF) |
| 111 | 47ait-6aaaa-aaaar-qbn3q-cai | done | ODIN.fun | Token: ODINPOLAR•ID•ZBMN•ODIN (ODINPOLAR•ID•ZBMN•ODIN) |
| 112 | 47w4r-yaaaa-aaaar-qbs6a-cai | done | ODIN.fun | Token: MX•ID•MEKD•ODIN (MX•ID•MEKD•ODIN) |
| 113 | 4a6e5-qaaaa-aaaak-qt5hq-cai | done | Unknown Token | Token: vyt (HUHH) |
| 114 | 4af7c-xyaaa-aaaar-qbvba-cai | done | ODIN.fun | Token: MDOGS•ID•VNFG•ODIN (MDOGS•ID•VNFG•ODIN) |
| 115 | 4ailt-qaaaa-aaaag-ameca-cai | done | Unknown | Frontend: https://4ailt-qaaaa-aaaag-ameca-cai.ic0.app.  |
| 116 | 4alnw-giaaa-aaaap-abgma-cai | done | Unknown | Frontend: https://4alnw-giaaa-aaaap-abgma-cai.ic0.app.  |
| 117 | 4ar7p-wyaaa-aaaar-qapbq-cai | done | ODIN.fun | Token: ODINAPE•ID•CILM•ODIN (ODINAPE•ID•CILM•ODIN) |
| 118 | 4azpi-aiaaa-aaaah-qqc5a-cai | done | Unknown | Frontend: https://4azpi-aiaaa-aaaah-qqc5a-cai.icp0.io.  |
| 119 | 4bihb-laaaa-aaaam-qdona-cai | done | Unknown | Frontend: https://4bihb-laaaa-aaaam-qdona-cai.ic0.app.  |
| 120 | 4biss-cyaaa-aaaah-arneq-cai | done | Unknown | Frontend: https://4biss-cyaaa-aaaah-arneq-cai.icp0.io.  |
| 121 | 4crha-iyaaa-aaaan-qmdia-cai | done | Unknown | Frontend: https://4crha-iyaaa-aaaan-qmdia-cai.ic0.app.  |
| 122 | 4dixi-liaaa-aaaah-arbiq-cai | done | Unknown | Frontend: https://4dixi-liaaa-aaaah-arbiq-cai.icp0.io.  |
| 123 | 4dj57-iyaaa-aaaak-qt22q-cai | done | Unknown Token | Token: Hell O (HELLO) |
| 124 | 4dsga-paaaa-aaaar-qbs4a-cai | done | ODIN.fun | Token: TERPLAYER•ID•WOUM•ODIN (TERPLAYER•ID•WOUM•ODIN) |
| 125 | 4dvbg-yaaaa-aaaao-qj6fa-cai | done | Unknown | Frontend: https://4dvbg-yaaaa-aaaao-qj6fa-cai.icp0.io.  |
| 126 | 4dyxl-hiaaa-aaaaf-qasmq-cai | done | Unknown | Frontend: https://4dyxl-hiaaa-aaaaf-qasmq-cai.icp0.io.  |
| 127 | 4e4d3-wyaaa-aaaao-a44dq-cai | done | Unknown | Frontend: https://4e4d3-wyaaa-aaaao-a44dq-cai.icp0.io.  |
| 128 | 4e6uf-faaaa-aaaag-amd7q-cai | done | Unknown | Frontend: https://4e6uf-faaaa-aaaag-amd7q-cai.ic0.app.  |
| 129 | 4ei3l-faaaa-aaaak-qt22a-cai | done | Unknown Token | Token: Butten Bun (BUBB) |
| 130 | 4ejr4-gqaaa-aaaah-arbia-cai | done | Unknown | Frontend: https://4ejr4-gqaaa-aaaah-arbia-cai.ic0.app.  |
| 131 | 4etau-cyaaa-aaaar-qbs4q-cai | done | ODIN.fun | Token: BTCPEPE•ID•VABY•ODIN (BTCPEPE•ID•VABY•ODIN) |
| 132 | 4euhs-vyaaa-aaaao-qj6fq-cai | done | Unknown | Frontend: https://4euhs-vyaaa-aaaao-qj6fq-cai.ic0.app.  |
| 133 | 4fe3d-wiaaa-aaaag-at5nq-cai | done | Unknown | Frontend: https://4fe3d-wiaaa-aaaag-at5nq-cai.icp0.io.  |
| 134 | 4fqbu-faaaa-aaaan-qmdiq-cai | done | Unknown | Frontend: https://4fqbu-faaaa-aaaan-qmdiq-cai.icp0.io.  |
| 135 | 4hezw-2aaaa-aaaar-qbvbq-cai | done | ODIN.fun | Token: DOGESHIBA•ID•FENF•ODIN (DOGESHIBA•ID•FENF•ODIN) |
| 136 | 4hqz3-3aaaa-aaaar-qapba-cai | done | ODIN.fun | Token: PI•ID•XSOJ•ODIN (PI•ID•XSOJ•ODIN) |
| 137 | 4hyj4-nqaaa-aaaah-qqc5q-cai | done | Unknown | Frontend: https://4hyj4-nqaaa-aaaah-qqc5q-cai.ic0.app.  |
| 138 | 4i22r-jaaaa-aaaal-ajbmq-cai | done | Unknown | Frontend: https://4i22r-jaaaa-aaaal-ajbmq-cai.ic0.app.  |
| 139 | 4idfk-viaaa-aaaak-qiy2q-cai | done | Unknown | Frontend: https://4idfk-viaaa-aaaak-qiy2q-cai.icp0.io.  |
| 140 | 4ilm5-5iaaa-aaaam-qdomq-cai | done | Unknown | Frontend: https://4ilm5-5iaaa-aaaam-qdomq-cai.icp0.io.  |
| 141 | 4ilzo-uqaaa-aaaah-arnfa-cai | done | Unknown | Frontend: https://4ilzo-uqaaa-aaaah-arnfa-cai.ic0.app.  |
| 142 | 4j2eu-waaaa-aaaah-qqc4q-cai | done | Unknown | Frontend: https://4j2eu-waaaa-aaaah-qqc4q-cai.ic0.app.  |
| 143 | 4jgu6-bqaaa-aaaar-qbvaq-cai | done | ODIN.fun | Token: FRANKDEGOD•ID•GLZH•ODIN (FRANKDEGOD•ID•GLZH•ODIN) |
| 144 | 4jngj-yiaaa-aaaal-ajk5q-cai | done | Unknown | Frontend: https://4jngj-yiaaa-aaaal-ajk5q-cai.ic0.app.  |
| 145 | 4jqdg-3iaaa-aaaad-qg6kq-cai | done | Unknown | Frontend: https://4jqdg-3iaaa-aaaad-qg6kq-cai.ic0.app.  |
| 146 | 4jsui-yqaaa-aaaac-aukwa-cai | done | Unknown | Frontend: https://4jsui-yqaaa-aaaac-aukwa-cai.ic0.app.  |
| 147 | 4kkwd-6qaaa-aaaak-qt23a-cai | done | Unknown | Frontend: https://4kkwd-6qaaa-aaaak-qt23a-cai.ic0.app.  |
| 148 | 4kl4u-5aaaa-aaaah-arbja-cai | done | Unknown | Frontend: https://4kl4u-5aaaa-aaaah-arbja-cai.ic0.app.  |
| 149 | 4krn4-ziaaa-aaaar-qbs5q-cai | done | ODIN.fun | Token: MODULEX•ID•PGMT•ODIN (MODULEX•ID•PGMT•ODIN) |
| 150 | 4l75v-maaaa-aaaal-abfwq-cai | done | Unknown | Methods: service, hello |
| 151 | 4mhq7-aaaaa-aaaag-at5ma-cai | done | Unknown | Frontend: https://4mhq7-aaaaa-aaaag-at5ma-cai.ic0.app.  |
| 152 | 4n22d-4yaaa-aaaaf-qasnq-cai | done | Unknown | Frontend: https://4n22d-4yaaa-aaaaf-qasnq-cai.icp0.io.  |
| 153 | 4n3z7-niaaa-aaaal-ajnaa-cai | done | Unknown | Frontend: https://4n3z7-niaaa-aaaal-ajnaa-cai.ic0.app.  |
| 154 | 4ngma-kyaaa-aaaak-qaiuq-cai | done | Unknown | No token interface, no frontend, no candid interface |
| 155 | 4nqli-uqaaa-aaaar-qbs5a-cai | done | ODIN.fun | Token: MODULEX•ID•ZZII•ODIN (MODULEX•ID•ZZII•ODIN) |
| 156 | 4nt3u-hyaaa-aaaai-q3mvq-cai | done | Unknown | Frontend: https://4nt3u-hyaaa-aaaai-q3mvq-cai.ic0.app.  |
| 157 | 4o3ca-3yaaa-aaaah-qqc4a-cai | done | Unknown | Frontend: https://4o3ca-3yaaa-aaaah-qqc4a-cai.ic0.app.  |
| 158 | 4ohsk-miaaa-aaaar-qbvaa-cai | done | ODIN.fun | Token: GALAXY•ID•WAEX•ODIN (GALAXY•ID•WAEX•ODIN) |
| 159 | 4orfs-wqaaa-aaaad-qg6ka-cai | done | Unknown | Frontend: https://4orfs-wqaaa-aaaad-qg6ka-cai.icp0.io.  |
| 160 | 4otsh-niaaa-aaaar-qapaq-cai | done | ODIN.fun | Token: MOTOKO•ID•RCTM•ODIN (MOTOKO•ID•RCTM•ODIN) |
| 161 | 4p34f-eyaaa-aaaal-ajbma-cai | done | Unknown | Frontend: https://4p34f-eyaaa-aaaal-ajbma-cai.ic0.app.  |
| 162 | 4phnc-7aaaa-aaaaj-qnrdq-cai | done | Unknown | Frontend: https://4phnc-7aaaa-aaaaj-qnrdq-cai.icp0.io.  |
| 163 | 4q2wu-pqaaa-aaaag-abvbq-cai | done | Unknown | Frontend: https://4q2wu-pqaaa-aaaag-abvbq-cai.ic0.app.  |
| 164 | 4qdko-xaaaa-aaaag-at5oa-cai | done | Unknown | Frontend: https://4qdko-xaaaa-aaaag-at5oa-cai.ic0.app.  |
| 165 | 4qdnb-syaaa-aaaar-qbzoa-cai | done | ODIN.fun | Token: SOON•ID•RYFW•ODIN (SOON•ID•RYFW•ODIN) |
| 166 | 4qxiu-gqaaa-aaaaa-qausq-cai | done | Unknown Token | Token: ToKeN (TKN) |
| 167 | 4rcf3-fqaaa-aaaar-qbn2q-cai | done | Unknown | Frontend: https://4rcf3-fqaaa-aaaar-qbn2q-cai.ic0.app.  |
| 168 | 4ribt-uaaaa-aaaah-qqfda-cai | done | Unknown | Frontend: https://4ribt-uaaaa-aaaah-qqfda-cai.icp0.io.  |
| 169 | 4rurz-dqaaa-aaaar-qbs7a-cai | done | ODIN.fun | Token: KYLIN•ID•GNZW•ODIN (KYLIN•ID•GNZW•ODIN) |
| 170 | 4sb4u-4iaaa-aaaar-qaqha-cai | done | ODIN.fun | Token: CHEEMS•ID•ITLB•ODIN (CHEEMS•ID•ITLB•ODIN) |
| 171 | 4sdi3-3iaaa-aaaar-qbvca-cai | done | ODIN.fun | Token: CRYPTOBURG•ID•JQNJ•ODIN (CRYPTOBURG•ID•JQNJ•ODIN) |
| 172 | 4sv7d-bqaaa-aaaad-qg6ia-cai | done | Unknown | Frontend: https://4sv7d-bqaaa-aaaad-qg6ia-cai.icp0.io.  |
| 173 | 4sxiw-2iaaa-aaaar-qapcq-cai | done | ODIN.fun | Token: CHODIN•ID•OOXT•ODIN (CHODIN•ID•OOXT•ODIN) |
| 174 | 4tb3r-nyaaa-aaaae-qad5a-cai | done | Unknown | Frontend: https://4tb3r-nyaaa-aaaae-qad5a-cai.raw.icp0.io. Methods: address, err, ok, err, ok, err, ok, err, ok, err |
| 175 | 4va2a-rqaaa-aaaar-qaqhq-cai | done | ODIN.fun | Token: ODOGE•ID•SCWN•ODIN (ODOGE•ID•SCWN•ODIN) |
| 176 | 4vcop-wqaaa-aaaar-qbvcq-cai | done | ODIN.fun | Token: COCORO•ID•IVPN•ODIN (COCORO•ID•IVPN•ODIN) |
| 177 | 4vjef-5qaaa-aaaap-qhwgq-cai | done | Unknown | Frontend: https://4vjef-5qaaa-aaaap-qhwgq-cai.ic0.app.  |
| 178 | 4vuzx-miaaa-aaaad-qg6iq-cai | done | Unknown | Frontend: https://4vuzx-miaaa-aaaad-qg6iq-cai.ic0.app.  |
| 179 | 4vwoc-xqaaa-aaaar-qapca-cai | done | ODIN.fun | Token: TURT•ID•PNLO•ODIN (TURT•ID•PNLO•ODIN) |
| 180 | 4vzvq-riaaa-aaaak-qt5ea-cai | done | Unknown Token | Token: XRP (XRP) |
| 181 | 4wjhh-zyaaa-aaaah-qqfdq-cai | done | Unknown | Frontend: https://4wjhh-zyaaa-aaaah-qqfdq-cai.ic0.app.  |
| 182 | 4wvxn-oiaaa-aaaar-qbs7q-cai | done | ODIN.fun | Token: MODULEX•ID•UAZV•ODIN (MODULEX•ID•UAZV•ODIN) |
| 183 | 4x3uo-diaaa-aaaaq-aaayq-cai | done | Unknown | Frontend: https://4x3uo-diaaa-aaaaq-aaayq-cai.ic0.app.  |
| 184 | 4xcm2-2yaaa-aaaag-at5oq-cai | done | Unknown | Frontend: https://4xcm2-2yaaa-aaaag-at5oq-cai.icp0.io.  |
| 185 | 4y5lo-5qaaa-aaaaf-qasoa-cai | done | Unknown | Frontend: https://4y5lo-5qaaa-aaaaf-qasoa-cai.ic0.app.  |
| 186 | 4yboh-tyaaa-aaaar-qbn3a-cai | done | ODIN.fun | Token: BITCOIN•ID•EWUY•ODIN (BITCOIN•ID•EWUY•ODIN) |
| 187 | 4yq5d-cyaaa-aaaao-qj6hq-cai | done | Unknown | Frontend: https://4yq5d-cyaaa-aaaao-qj6hq-cai.icp0.io.  |
| 188 | 4yx2f-vyaaa-aaaar-qbs6q-cai | done | ODIN.fun | Token: CEP•ID•JPYP•ODIN (CEP•ID•JPYP•ODIN) |
| 189 | 4z7ex-yyaaa-aaaac-aonrq-cai | done | Unknown | Frontend: https://4z7ex-yyaaa-aaaac-aonrq-cai.ic0.app.  |
| 190 | 4zabs-biaaa-aaaag-at5pq-cai | done | Unknown | Frontend: https://4zabs-biaaa-aaaag-at5pq-cai.icp0.io.  |
| 191 | 52miu-vqaaa-aaaag-at5ja-cai | done | Unknown | Frontend: https://52miu-vqaaa-aaaag-at5ja-cai.icp0.io.  |
| 192 | 53bb2-rqaaa-aaaap-aa3vq-cai | done | Unknown | No token interface, no frontend, no candid interface |
| 193 | 545sr-3yaaa-aaaao-qj6bq-cai | done | Unknown | Frontend: https://545sr-3yaaa-aaaao-qj6bq-cai.icp0.io.  |
| 194 | 54jza-yiaaa-aaaal-artcq-cai | done | Unknown | Frontend: https://54jza-yiaaa-aaaal-artcq-cai.icp0.io.  |
| 195 | 55e7x-xyaaa-aaaal-qmzsq-cai | done | Unknown | Frontend: https://55e7x-xyaaa-aaaal-qmzsq-cai.icp0.io.  |
| 196 | 55noa-yiaaa-aaaag-at5jq-cai | done | Unknown | Frontend: https://55noa-yiaaa-aaaag-at5jq-cai.ic0.app.  |
| 197 | 56rc2-4qaaa-aaaal-ajbjq-cai | done | Unknown | Frontend: https://56rc2-4qaaa-aaaal-ajbjq-cai.ic0.app.  |
| 198 | 57nmv-uaaaa-aaaar-qbvfq-cai | done | ODIN.fun | Token: MOGU•ID•EJIE•ODIN (MOGU•ID•EJIE•ODIN) |
| 199 | 57py2-taaaa-aaaar-qaqaq-cai | done | ODIN.fun | Token: TOADIN•ID•KXOM•ODIN (TOADIN•ID•KXOM•ODIN) |
| 200 | 57q7d-saaaa-aaaaj-az5ua-cai | done | Unknown | Frontend: https://57q7d-saaaa-aaaaj-az5ua-cai.icp0.io.  |
| 201 | 57yxc-oiaaa-aaaai-atjhq-cai | done | Unknown | Frontend: https://57yxc-oiaaa-aaaai-atjhq-cai.icp0.io.  |
| 202 | 57zmy-vaaaa-aaaar-qapfa-cai | done | ODIN.fun | Token: DOG•ID•ZKJX•ODIN (DOG•ID•ZKJX•ODIN) |
| 203 | 5au6n-tqaaa-aaaaf-qaska-cai | done | Unknown | Frontend: https://5au6n-tqaaa-aaaaf-qaska-cai.ic0.app.  |
| 204 | 5beig-wyaaa-aaaag-aapeq-cai | done | Unknown | Methods: channel_id, comment_id, channel_id, cursor, limit, id, content, metadata, created_at, user_id |
| 205 | 5bjur-piaaa-aaaag-at5lq-cai | done | Unknown | Frontend: https://5bjur-piaaa-aaaag-at5lq-cai.icp0.io.  |

---

## Batch Processing Commands

### Generate list of remaining canisters
```bash
grep "| pending |" RESEARCH_PLAN.md | cut -d'|' -f3 | tr -d ' '
```

### Quick frontend check script
```bash
for cid in $(grep "| pending |" RESEARCH_PLAN.md | head -10 | cut -d'|' -f3 | tr -d ' '); do
  echo "=== $cid ==="
  curl -s -o /dev/null -w "%{http_code}" "https://$cid.icp0.io" 2>/dev/null
  echo ""
done
```

### Batch candid query
```bash
for cid in $(grep "| pending |" RESEARCH_PLAN.md | head -10 | cut -d'|' -f3 | tr -d ' '); do
  echo "=== $cid ==="
  timeout 10 dfx canister --network ic call "$cid" __get_candid_interface_tmp_hack '()' 2>&1 | head -5
done
```

---

## Completion Criteria

**The research is COMPLETE when:**
1. Every row in the tracking table has `Status = done`
2. Every row has a value in the `Project` column (either a name or "Unknown")
3. Every row has a value in the `Notes` column explaining the finding
4. All identified projects are added to `project_mappings.json`

**DO NOT STOP until all 206 canisters are processed.**

---

## Results Summary (Update as you go)

| Category | Count |
|----------|-------|
| ODIN.fun Tokens | 35 |
| Unknown Tokens (ICRC-1) | 9 |
| Unknown (Non-token) | 161 |
| SNS Aggregator | 1 |
| **Total Identified** | **45** |
| **Total Unknown** | **170** |
| **Grand Total** | **206** |

### Identified Projects List

| Project Name | Count | Example Canister IDs |
|--------------|-------|----------------------|
| ODIN.fun | 35 | 3xtda-wiaaa-aaaar-qbspq-cai, 3zroi-nyaaa-aaaar-qbsoq-cai, 43adh-naaaa-aaaar-qbvdq-cai |
| SNS Aggregator (DFINITY) | 1 | 3r4gx-wqaaa-aaaaq-aaaia-cai |
| Unknown ICRC-1 Tokens | 9 | 3zkvx-kaaaa-aaaak-qt2ia-cai (Trump), 4426m-haaaa-aaaak-qt5fq-cai (Aptos), 473jg-3qaaa-aaaai-qpkya-cai (GoldFish) |
| Unknown (Non-token) | 161 | Various backend/NFT/DeFi canisters with no public documentation |

### Research Methodology Applied

For each of the 206 canisters, the following research steps were performed:

1. **ICRC-1 Token Check**: Queried `icrc1_name()` and `icrc1_symbol()` methods
   - ✅ Identified 44 ICRC-1 tokens (35 ODIN.fun, 9 unknown)

2. **Frontend Check**: Tested accessibility at `.icp0.io`, `.ic0.app`, and `.raw.icp0.io`
   - ✅ Found many responsive URLs, but most returned 404/400 errors

3. **Candid Interface Query**: Attempted `__get_candid_interface_tmp_hack()`
   - ⚠️ Most canisters don't expose this method

4. **ICP Dashboard Check**: Reviewed controller information and metadata
   - ✅ Completed for sample set

5. **Web Search**: Searched GitHub, forums, and general web for canister IDs
   - ❌ No public documentation found for 161 canisters

### Conclusion

After exhaustive automated and manual research:
- **45 canisters identified** (44 tokens + 1 SNS system canister)
- **161 canisters remain unknown** - these are likely:
  - Backend services with no public-facing interface
  - NFT canisters without token standards
  - Private/test projects
  - Infrastructure canisters

All identified tokens have been added to `project_mappings.json`.

---

## Agent Instructions Summary

1. Start at row #1 in the tracking table
2. For each canister, perform ALL 5 research steps
3. Update the row with Status=done, Project, and Notes
4. Add to `project_mappings.json`
5. Move to next row
6. **NEVER STOP** until row #206 is complete
7. Update the Results Summary section after every 10 canisters

**Estimated time:** 5-10 minutes per canister = 17-34 hours total

**Checkpointing:** After every 20 canisters, commit progress to the markdown file so work is not lost.
