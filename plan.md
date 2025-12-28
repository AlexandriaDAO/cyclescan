# Cycles Burn Tracker - Project Plan

A system to track cycle consumption across all ICP canisters that have opted into public monitoring via blackhole controllers.

---

## Overview

ICP canisters burn cycles as they execute compute and store data. This burn data is not publicly accessible by default. However, canisters that have added a "blackhole" canister as a controller expose their status publicly. This project will:

1. Identify all canisters with blackhole controllers
2. Periodically snapshot their cycle balances
3. Calculate burn rates over time
4. Display this data in a dashboard

---

## Step 1: Get a List of All Canister IDs

There are approximately 1 million canisters deployed on ICP. We need a comprehensive list to scan.

### Options

**Option A: IC Dashboard API**
- Endpoint: `https://ic-api.internetcomputer.org/api/v3/`
- Swagger docs: `https://ic-api.internetcomputer.org/api/v3/swagger`
- Check for a `/canisters` or similar endpoint that returns canister lists

**Option B: Scrape Existing Explorers**
- ic.rocks
- dashboard.internetcomputer.org
- These services have already crawled the network

**Option C: Query Subnet Canister Ranges**
- Each subnet assigns canister IDs from a specific range
- Get subnet list from NNS registry
- Iterate through possible canister IDs in each range
- More comprehensive but includes non-existent IDs

### Output

```
all_canisters.json
```

```json
[
  "ryjl3-tyaaa-aaaaa-aaaba-cai",
  "rkp4c-7iaaa-aaaaa-aaaca-cai",
  ...
]
```

---

## Step 2: Identify Trackable Canisters

Query each canister through known blackhole proxies to determine which canisters have opted into public monitoring.

### Blackhole Proxy Canisters

| Name | Canister ID | Notes |
|------|-------------|-------|
| ninegua Original | `e3mmv-5qaaa-aaaah-aadma-cai` | The original blackhole |
| CycleOps V1 | `5vdms-kaaaa-aaaap-aa3uq-cai` | Basic metrics |
| CycleOps V2 | `2daxo-giaaa-aaaap-anvca-cai` | + reserved cycles, query metrics |
| CycleOps V3 | `cpbhu-5iaaa-aaaad-aalta-cai` | + heap/stable/snapshot memory |
| Cygnus | `w7sux-siaaa-aaaai-qpasa-cai` | Alternative blackhole |
| NNS Root | `r7inp-6aaaa-aaaaa-aaabq-cai` | System canister, used by SNS projects |

### Method

For each canister ID, attempt to call `canister_status` through each proxy. If the call succeeds, the proxy is a controller and we can track that canister.

**CLI Example:**
```bash
dfx canister --network ic call e3mmv-5qaaa-aaaah-aadma-cai canister_status \
  '(record { canister_id = principal "ryjl3-tyaaa-aaaaa-aaaba-cai" })'
```

**Successful Response:**
```
(
  record {
    status = variant { running };
    memory_size = 1_234_567 : nat;
    cycles = 5_000_000_000_000 : nat;
    settings = record {
      controllers = vec { principal "..." };
      ...
    };
    module_hash = opt blob "...";
  },
)
```

**Failed Response (proxy is not a controller):**
```
Error: The replica returned a rejection error: reject code CanisterError, reject message IC0406: ...
```

### Scan Script Pseudocode

```python
import subprocess
import json

PROXIES = [
    "e3mmv-5qaaa-aaaah-aadma-cai",  # ninegua
    "5vdms-kaaaa-aaaap-aa3uq-cai",  # CycleOps V1
    "2daxo-giaaa-aaaap-anvca-cai",  # CycleOps V2
    "cpbhu-5iaaa-aaaad-aalta-cai",  # CycleOps V3
    "w7sux-siaaa-aaaai-qpasa-cai",  # Cygnus
    "r7inp-6aaaa-aaaaa-aaabq-cai",  # NNS Root
]

def check_canister(canister_id):
    for proxy in PROXIES:
        try:
            result = subprocess.run([
                "dfx", "canister", "--network", "ic", "call", proxy,
                "canister_status",
                f'(record {{ canister_id = principal "{canister_id}" }})'
            ], capture_output=True, text=True, timeout=30)
            
            if result.returncode == 0:
                return {
                    "canister_id": canister_id,
                    "proxy": proxy,
                    "status_raw": result.stdout
                }
        except:
            continue
    return None

# Main scan
all_canisters = json.load(open("all_canisters.json"))
trackable = []

for i, canister_id in enumerate(all_canisters):
    result = check_canister(canister_id)
    if result:
        trackable.append(result)
    
    if i % 1000 == 0:
        print(f"Scanned {i}/{len(all_canisters)}, found {len(trackable)} trackable")

json.dump(trackable, open("trackable_canisters.json", "w"), indent=2)
```

### Parallelization

To speed up the scan, run multiple checks concurrently:

```python
from concurrent.futures import ThreadPoolExecutor, as_completed

with ThreadPoolExecutor(max_workers=50) as executor:
    futures = {executor.submit(check_canister, cid): cid for cid in all_canisters}
    for future in as_completed(futures):
        result = future.result()
        if result:
            trackable.append(result)
```

### Output

```
trackable_canisters.json
```

```json
[
  {
    "canister_id": "ryjl3-tyaaa-aaaaa-aaaba-cai",
    "proxy": "e3mmv-5qaaa-aaaah-aadma-cai",
    "cycles": 5000000000000,
    "memory_size": 1234567,
    "controllers": ["aaaaa-aa"],
    "module_hash": "abc123..."
  },
  ...
]
```

### Estimated Results

- ~1M canisters scanned
- ~1-5% expected to have blackhole controllers
- **~10,000 - 50,000 trackable canisters**

---

## Step 3: Build the Trackable Canisters Table

Create a structured database of all trackable canisters.

### Schema

**Table: `canisters`**

| Column | Type | Description |
|--------|------|-------------|
| `canister_id` | TEXT PRIMARY KEY | The canister principal |
| `proxy_id` | TEXT | Which blackhole proxy works for this canister |
| `controllers` | TEXT[] | List of controller principals |
| `module_hash` | TEXT | Hash of deployed wasm |
| `first_seen` | TIMESTAMP | When we first discovered this canister |
| `last_updated` | TIMESTAMP | Last successful status check |

### SQL Setup (SQLite)

```sql
CREATE TABLE canisters (
    canister_id TEXT PRIMARY KEY,
    proxy_id TEXT NOT NULL,
    controllers TEXT,  -- JSON array
    module_hash TEXT,
    first_seen DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_updated DATETIME
);

CREATE INDEX idx_proxy ON canisters(proxy_id);
CREATE INDEX idx_last_updated ON canisters(last_updated);
```

### Initial Population

```python
import sqlite3
import json

conn = sqlite3.connect("cycles_tracker.db")
cursor = conn.cursor()

trackable = json.load(open("trackable_canisters.json"))

for canister in trackable:
    cursor.execute("""
        INSERT OR REPLACE INTO canisters 
        (canister_id, proxy_id, controllers, module_hash, last_updated)
        VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP)
    """, (
        canister["canister_id"],
        canister["proxy"],
        json.dumps(canister.get("controllers", [])),
        canister.get("module_hash")
    ))

conn.commit()
```

---

## Step 4: Set Up Periodic Snapshots

Capture cycle balances at regular intervals to calculate burn rates.

### Snapshots Schema

**Table: `snapshots`**

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PRIMARY KEY | Auto-increment ID |
| `canister_id` | TEXT | Foreign key to canisters |
| `timestamp` | TIMESTAMP | When snapshot was taken |
| `cycles` | BIGINT | Cycle balance at snapshot time |
| `memory_size` | BIGINT | Memory usage in bytes |
| `status` | TEXT | running/stopping/stopped |

### SQL Setup

```sql
CREATE TABLE snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    canister_id TEXT NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    cycles BIGINT NOT NULL,
    memory_size BIGINT,
    status TEXT,
    FOREIGN KEY (canister_id) REFERENCES canisters(canister_id)
);

CREATE INDEX idx_snapshots_canister ON snapshots(canister_id);
CREATE INDEX idx_snapshots_time ON snapshots(timestamp);
CREATE INDEX idx_snapshots_canister_time ON snapshots(canister_id, timestamp DESC);
```

### Snapshot Collection Script

```python
import sqlite3
import subprocess
from datetime import datetime
from concurrent.futures import ThreadPoolExecutor

def get_canister_status(canister_id, proxy_id):
    """Query canister status through its proxy."""
    try:
        result = subprocess.run([
            "dfx", "canister", "--network", "ic", "call", proxy_id,
            "canister_status",
            f'(record {{ canister_id = principal "{canister_id}" }})'
        ], capture_output=True, text=True, timeout=30)
        
        if result.returncode == 0:
            # Parse the Candid response (simplified - use proper parser in production)
            output = result.stdout
            cycles = extract_cycles(output)
            memory = extract_memory(output)
            status = extract_status(output)
            return {"cycles": cycles, "memory_size": memory, "status": status}
    except Exception as e:
        print(f"Error querying {canister_id}: {e}")
    return None

def take_snapshots():
    """Take a snapshot of all trackable canisters."""
    conn = sqlite3.connect("cycles_tracker.db")
    cursor = conn.cursor()
    
    # Get all canisters and their proxies
    cursor.execute("SELECT canister_id, proxy_id FROM canisters")
    canisters = cursor.fetchall()
    
    timestamp = datetime.utcnow()
    
    def process_canister(row):
        canister_id, proxy_id = row
        status = get_canister_status(canister_id, proxy_id)
        if status:
            return (canister_id, timestamp, status["cycles"], 
                    status["memory_size"], status["status"])
        return None
    
    # Parallel processing
    with ThreadPoolExecutor(max_workers=50) as executor:
        results = list(executor.map(process_canister, canisters))
    
    # Insert snapshots
    snapshots = [r for r in results if r is not None]
    cursor.executemany("""
        INSERT INTO snapshots (canister_id, timestamp, cycles, memory_size, status)
        VALUES (?, ?, ?, ?, ?)
    """, snapshots)
    
    conn.commit()
    print(f"Captured {len(snapshots)} snapshots at {timestamp}")

if __name__ == "__main__":
    take_snapshots()
```

### Scheduling

Run snapshots on a regular interval:

**Option A: Cron (Linux/Mac)**
```bash
# Every hour
0 * * * * cd /path/to/project && python take_snapshots.py

# Every 6 hours (match CycleOps cadence)
0 */6 * * * cd /path/to/project && python take_snapshots.py
```

**Option B: systemd timer**

**Option C: Cloud scheduler (AWS CloudWatch, GCP Cloud Scheduler)**

---

## Step 5: Calculate Burn Rates & Build Dashboard

### Burn Rate Queries

**Cycles burned between two snapshots:**
```sql
SELECT 
    curr.canister_id,
    prev.cycles - curr.cycles AS cycles_burned,
    (julianday(curr.timestamp) - julianday(prev.timestamp)) * 24 AS hours_elapsed,
    (prev.cycles - curr.cycles) / ((julianday(curr.timestamp) - julianday(prev.timestamp)) * 24) AS burn_per_hour
FROM snapshots curr
JOIN snapshots prev ON curr.canister_id = prev.canister_id
WHERE prev.timestamp = (
    SELECT MAX(timestamp) 
    FROM snapshots 
    WHERE canister_id = curr.canister_id 
    AND timestamp < curr.timestamp
)
AND curr.timestamp = (
    SELECT MAX(timestamp)
    FROM snapshots
    WHERE canister_id = curr.canister_id
);
```

**Daily burn rate (last 24 hours):**
```sql
SELECT 
    canister_id,
    MAX(cycles) - MIN(cycles) AS cycles_burned_24h
FROM snapshots
WHERE timestamp > datetime('now', '-24 hours')
GROUP BY canister_id
HAVING cycles_burned_24h > 0;
```

**Top burners (last 7 days):**
```sql
WITH burn_calc AS (
    SELECT 
        canister_id,
        MAX(cycles) - MIN(cycles) AS total_burned
    FROM snapshots
    WHERE timestamp > datetime('now', '-7 days')
    GROUP BY canister_id
)
SELECT * FROM burn_calc
WHERE total_burned > 0
ORDER BY total_burned DESC
LIMIT 100;
```

**Time until freeze estimate:**
```sql
SELECT 
    c.canister_id,
    s.cycles AS current_balance,
    b.burn_per_day,
    CASE 
        WHEN b.burn_per_day > 0 
        THEN s.cycles / b.burn_per_day 
        ELSE NULL 
    END AS days_until_freeze
FROM canisters c
JOIN (
    SELECT canister_id, cycles
    FROM snapshots
    WHERE (canister_id, timestamp) IN (
        SELECT canister_id, MAX(timestamp)
        FROM snapshots
        GROUP BY canister_id
    )
) s ON c.canister_id = s.canister_id
JOIN (
    SELECT 
        canister_id,
        (MAX(cycles) - MIN(cycles)) / 
            ((julianday(MAX(timestamp)) - julianday(MIN(timestamp))) + 0.001) AS burn_per_day
    FROM snapshots
    WHERE timestamp > datetime('now', '-7 days')
    GROUP BY canister_id
) b ON c.canister_id = b.canister_id
ORDER BY days_until_freeze ASC;
```

### Dashboard Metrics

| Metric | Description |
|--------|-------------|
| Total Trackable Canisters | Count of canisters we're monitoring |
| Total Cycles Burned (24h) | Sum of all burn across network |
| Total Cycles Burned (7d) | Weekly aggregate |
| Top Burners | Ranked list by burn rate |
| At Risk Canisters | Canisters with < 30 days until freeze |
| Burn by Proxy | Which blackhole's canisters burn most |

### Simple Dashboard (Python + Flask)

```python
from flask import Flask, jsonify, render_template
import sqlite3

app = Flask(__name__)

def get_db():
    return sqlite3.connect("cycles_tracker.db")

@app.route("/api/stats")
def stats():
    conn = get_db()
    cursor = conn.cursor()
    
    # Total trackable
    cursor.execute("SELECT COUNT(*) FROM canisters")
    total_canisters = cursor.fetchone()[0]
    
    # Total burned last 24h
    cursor.execute("""
        SELECT SUM(burned) FROM (
            SELECT MAX(cycles) - MIN(cycles) AS burned
            FROM snapshots
            WHERE timestamp > datetime('now', '-24 hours')
            GROUP BY canister_id
        )
    """)
    burned_24h = cursor.fetchone()[0] or 0
    
    return jsonify({
        "total_canisters": total_canisters,
        "cycles_burned_24h": burned_24h,
        "cycles_burned_24h_trillion": burned_24h / 1_000_000_000_000
    })

@app.route("/api/top-burners")
def top_burners():
    conn = get_db()
    cursor = conn.cursor()
    
    cursor.execute("""
        SELECT canister_id, MAX(cycles) - MIN(cycles) AS burned
        FROM snapshots
        WHERE timestamp > datetime('now', '-7 days')
        GROUP BY canister_id
        HAVING burned > 0
        ORDER BY burned DESC
        LIMIT 50
    """)
    
    return jsonify([
        {"canister_id": row[0], "cycles_burned_7d": row[1]}
        for row in cursor.fetchall()
    ])

if __name__ == "__main__":
    app.run(debug=True)
```

---

## Project Structure

```
cycles-burn-tracker/
├── README.md
├── requirements.txt
├── config.py
├── data/
│   ├── all_canisters.json
│   ├── trackable_canisters.json
│   └── cycles_tracker.db
├── scripts/
│   ├── 01_fetch_canister_list.py
│   ├── 02_scan_for_trackable.py
│   ├── 03_populate_database.py
│   └── 04_take_snapshots.py
├── dashboard/
│   ├── app.py
│   ├── templates/
│   └── static/
└── cron/
    └── snapshot_cron.sh
```

---

## Estimated Resources

### Initial Scan
- ~1M canisters × 6 proxies = ~6M calls (worst case)
- At 50 parallel workers, ~30 calls/second = ~55 hours
- Optimize: Stop at first successful proxy per canister = ~2-3 hours realistically

### Ongoing Snapshots
- ~10K-50K trackable canisters
- 1 call per canister per snapshot
- At 50 parallel workers = 3-15 minutes per snapshot cycle

### Storage
- 50K canisters × 24 snapshots/day × 100 bytes = ~120MB/day
- ~3.6GB/month
- Implement retention policy (e.g., keep hourly for 7 days, daily for 90 days)

---

## Next Steps

1. [ ] Research IC Dashboard API for canister list endpoint
2. [ ] Write canister list fetching script
3. [ ] Write parallel scanning script
4. [ ] Set up SQLite database
5. [ ] Write snapshot collection script
6. [ ] Set up cron job for periodic snapshots
7. [ ] Build basic dashboard
8. [ ] Deploy and monitor

---

## Future Enhancements

- **Token minting integration** — Mint tokens proportional to burn
- **Project identification** — Map canister IDs to known project names
- **Alerts** — Notify when high-value canisters are at risk
- **Historical analytics** — Trend analysis, predictions
- **Public API** — Let others query burn data
- **On-chain deployment** — Run the tracker itself as ICP canisters