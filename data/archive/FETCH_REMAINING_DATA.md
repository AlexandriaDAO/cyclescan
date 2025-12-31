# Task: Fetch All Remaining Canister Data

**Goal:** Fetch canister data from offset 300,000 to 1,000,000 and extract trackable canisters.

**Do not stop until complete. Do not ask questions. Just execute.**

---

## Current State

Already completed (or in progress):
- `canisters_0_25000.json` through `canisters_175000_200000.json` - DONE
- `canisters_200000_225000.json` through `canisters_275000_300000.json` - DONE (or finishing)

**You need to fetch: 300,000 to 1,000,000** (28 more batches of 25,000 each)

---

## Step 1: Navigate to data directory

```bash
cd /home/theseus/alexandria/cyclescan/data
```

---

## Step 2: Fetch batches in groups of 4

Run these commands. Each group runs 4 batches in parallel, then waits for completion before the next group.

**IMPORTANT:** Use `wait` to block until all 4 complete before starting the next group. This prevents overwhelming the API.

### Group 1: 200k-300k
```bash
./fetch_batch.sh 200000 225000 &
./fetch_batch.sh 225000 250000 &
./fetch_batch.sh 250000 275000 &
./fetch_batch.sh 275000 300000 &
wait
```

### Group 2: 300k-400k
```bash
./fetch_batch.sh 300000 325000 &
./fetch_batch.sh 325000 350000 &
./fetch_batch.sh 350000 375000 &
./fetch_batch.sh 375000 400000 &
wait
```

### Group 3: 400k-500k
```bash
./fetch_batch.sh 400000 425000 &
./fetch_batch.sh 425000 450000 &
./fetch_batch.sh 450000 475000 &
./fetch_batch.sh 475000 500000 &
wait
```

### Group 4: 500k-600k
```bash
./fetch_batch.sh 500000 525000 &
./fetch_batch.sh 525000 550000 &
./fetch_batch.sh 550000 575000 &
./fetch_batch.sh 575000 600000 &
wait
```

### Group 5: 600k-700k
```bash
./fetch_batch.sh 600000 625000 &
./fetch_batch.sh 625000 650000 &
./fetch_batch.sh 650000 675000 &
./fetch_batch.sh 675000 700000 &
wait
```

### Group 6: 700k-800k
```bash
./fetch_batch.sh 700000 725000 &
./fetch_batch.sh 725000 750000 &
./fetch_batch.sh 750000 775000 &
./fetch_batch.sh 775000 800000 &
wait
```

### Group 7: 800k-900k
```bash
./fetch_batch.sh 800000 825000 &
./fetch_batch.sh 825000 850000 &
./fetch_batch.sh 850000 875000 &
./fetch_batch.sh 875000 900000 &
wait
```

### Group 8: 900k-1M
```bash
./fetch_batch.sh 900000 925000 &
./fetch_batch.sh 925000 950000 &
./fetch_batch.sh 950000 975000 &
./fetch_batch.sh 975000 1000000 &
wait
```

---

## Step 3: Extract trackable canisters from each new file

After ALL fetches complete, run extraction on each new file:

```bash
for f in canisters_*.json; do
  if [ ! -f "public_$(basename $f .json | sed 's/canisters_/canisters_/')" ]; then
    ./extract_trackable.sh "$f"
  fi
done
```

Or explicitly:
```bash
./extract_trackable.sh canisters_200000_225000.json
./extract_trackable.sh canisters_225000_250000.json
./extract_trackable.sh canisters_250000_275000.json
./extract_trackable.sh canisters_275000_300000.json
./extract_trackable.sh canisters_300000_325000.json
./extract_trackable.sh canisters_325000_350000.json
./extract_trackable.sh canisters_350000_375000.json
./extract_trackable.sh canisters_375000_400000.json
./extract_trackable.sh canisters_400000_425000.json
./extract_trackable.sh canisters_425000_450000.json
./extract_trackable.sh canisters_450000_475000.json
./extract_trackable.sh canisters_475000_500000.json
./extract_trackable.sh canisters_500000_525000.json
./extract_trackable.sh canisters_525000_550000.json
./extract_trackable.sh canisters_550000_575000.json
./extract_trackable.sh canisters_575000_600000.json
./extract_trackable.sh canisters_600000_625000.json
./extract_trackable.sh canisters_625000_650000.json
./extract_trackable.sh canisters_650000_675000.json
./extract_trackable.sh canisters_675000_700000.json
./extract_trackable.sh canisters_700000_725000.json
./extract_trackable.sh canisters_725000_750000.json
./extract_trackable.sh canisters_750000_775000.json
./extract_trackable.sh canisters_775000_800000.json
./extract_trackable.sh canisters_800000_825000.json
./extract_trackable.sh canisters_825000_850000.json
./extract_trackable.sh canisters_850000_875000.json
./extract_trackable.sh canisters_875000_900000.json
./extract_trackable.sh canisters_900000_925000.json
./extract_trackable.sh canisters_925000_950000.json
./extract_trackable.sh canisters_950000_975000.json
./extract_trackable.sh canisters_975000_1000000.json
```

---

## Step 4: Combine all trackable canisters into one file

```bash
jq -s 'add | unique_by(.canister_id)' public_canisters_*.json > trackable_canisters.json
echo "Total trackable canisters:"
jq 'length' trackable_canisters.json
```

---

## Step 5: Verify completion

```bash
echo "=== Verification ==="
echo "Expected raw files: 40"
ls canisters_*.json | wc -l

echo "Expected public files: 40"
ls public_canisters_*.json | wc -l

echo "Total raw canisters:"
jq -s 'add | length' canisters_*.json

echo "Total trackable canisters:"
jq 'length' trackable_canisters.json
```

Expected results:
- 40 raw files (canisters_*.json)
- 40 public files (public_canisters_*.json)
- ~1,000,000 raw canisters
- ~3,000-4,000 trackable canisters

---

## Timing Expectations

- Each batch of 25,000 takes ~5-8 minutes
- Running 4 in parallel = ~8 minutes per group
- 8 groups = ~64 minutes for all fetches
- Extraction is fast (~30 seconds total)

**Total time: ~1-1.5 hours**

---

## If API errors occur

Some batches may have fewer than 25,000 due to API timeouts. This is acceptable. Do not retry - the data is still usable.

---

## Scripts Reference

### fetch_batch.sh
- Takes START and END offsets
- Fetches from IC Dashboard API in batches of 100
- Outputs `canisters_START_END.json`

### extract_trackable.sh
- Takes a raw canisters JSON file
- Filters for canisters with ninegua or NNS Root controllers
- Outputs `public_canisters_START_END.json`

---

## Final Deliverable

When complete, report:
1. Total raw canisters fetched
2. Total trackable canisters found
3. Confirm `trackable_canisters.json` exists with deduplicated combined data
