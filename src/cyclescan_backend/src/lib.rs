use candid::{CandidType, Nat, Principal};
use ic_cdk::api::call::CallResult;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cell::RefCell;

// =============================================================================
// Hardcoded Canister List (compiled into WASM)
// =============================================================================

const TRACKABLE_CANISTERS_JSON: &str = include_str!("../../../data/trackable_canisters.json");

/// Parsed canister entry from JSON
#[derive(Deserialize)]
struct JsonCanisterEntry {
    canister_id: String,
    proxy: String,
}

/// Get the list of trackable canisters (parsed once, cached)
fn get_trackable_canisters() -> Vec<(Principal, Principal)> {
    let entries: Vec<JsonCanisterEntry> =
        serde_json::from_str(TRACKABLE_CANISTERS_JSON).unwrap_or_default();

    entries
        .into_iter()
        .filter_map(|e| {
            let canister = Principal::from_text(&e.canister_id).ok()?;
            let proxy = Principal::from_text(&e.proxy).ok()?;
            Some((canister, proxy))
        })
        .collect()
}

// =============================================================================
// Types
// =============================================================================

/// A minimal snapshot: just canister_id, timestamp, cycles
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Snapshot {
    pub canister_id: Principal,
    pub timestamp: u64,
    pub cycles: u128,
}

/// Result of a snapshot operation
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct SnapshotResult {
    pub total_canisters: u64,
    pub successful: u64,
    pub failed: u64,
    pub timestamp: u64,
}

/// Burn rate calculation result
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct BurnRate {
    pub canister_id: Principal,
    pub cycles_per_day: u128,
    pub latest_balance: u128,
    pub days_until_empty: Option<f64>,
    pub snapshots_used: u64,
}

/// Canister info for queries
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct CanisterInfo {
    pub canister_id: Principal,
    pub proxy_id: Principal,
}

// =============================================================================
// Storable implementations for snapshots
// =============================================================================

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SnapshotKey {
    canister_id: [u8; 29],
    canister_len: u8,
    timestamp: u64,
}

impl SnapshotKey {
    fn new(canister_id: Principal, timestamp: u64) -> Self {
        let slice = canister_id.as_slice();
        let mut bytes = [0u8; 29];
        bytes[..slice.len()].copy_from_slice(slice);
        Self {
            canister_id: bytes,
            canister_len: slice.len() as u8,
            timestamp,
        }
    }

    fn to_principal(&self) -> Principal {
        Principal::from_slice(&self.canister_id[..self.canister_len as usize])
    }
}

impl Storable for SnapshotKey {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let mut bytes = Vec::with_capacity(38);
        bytes.push(self.canister_len);
        bytes.extend_from_slice(&self.canister_id);
        bytes.extend_from_slice(&self.timestamp.to_be_bytes());
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let canister_len = bytes[0];
        let mut canister_id = [0u8; 29];
        canister_id.copy_from_slice(&bytes[1..30]);
        let timestamp = u64::from_be_bytes(bytes[30..38].try_into().unwrap());
        Self {
            canister_id,
            canister_len,
            timestamp,
        }
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 38,
        is_fixed_size: true,
    };
}

#[derive(Clone)]
struct CyclesValue(u128);

impl Storable for CyclesValue {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(self.0.to_be_bytes().to_vec())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Self(u128::from_be_bytes(bytes.as_ref().try_into().unwrap()))
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 16,
        is_fixed_size: true,
    };
}

// =============================================================================
// Memory Management
// =============================================================================

type Memory = VirtualMemory<DefaultMemoryImpl>;

const SNAPSHOTS_MEM_ID: MemoryId = MemoryId::new(0);

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static SNAPSHOTS: RefCell<StableBTreeMap<SnapshotKey, CyclesValue, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(SNAPSHOTS_MEM_ID))
        ));
}

// =============================================================================
// Helpers
// =============================================================================

fn now_nanos() -> u64 {
    ic_cdk::api::time()
}

fn nat_to_u128(nat: &Nat) -> u128 {
    let bytes = nat.0.to_bytes_le();
    if bytes.len() > 16 {
        u128::MAX
    } else {
        let mut arr = [0u8; 16];
        arr[..bytes.len()].copy_from_slice(&bytes);
        u128::from_le_bytes(arr)
    }
}

// =============================================================================
// Canister Status Query via Blackhole
// =============================================================================

/// Request type for canister_status
#[derive(CandidType, Deserialize)]
struct CanisterIdRecord {
    canister_id: Principal,
}

/// Response type for canister_status - handles both ninegua and NNS Root formats
/// Uses Option for fields that may or may not be present
#[derive(CandidType, Deserialize, Debug)]
struct BlackholeCanisterStatus {
    status: BlackholeStatus,
    memory_size: Nat,
    cycles: Nat,
    settings: BlackholeSettings,
    #[serde(default)]
    module_hash: Option<Vec<u8>>,
    // NNS Root extra fields (optional)
    #[serde(default)]
    idle_cycles_burned_per_day: Option<Nat>,
    #[serde(default)]
    memory_metrics: Option<candid::Reserved>,
    #[serde(default)]
    query_stats: Option<candid::Reserved>,
    #[serde(default)]
    reserved_cycles: Option<Nat>,
}

#[derive(CandidType, Deserialize, Debug)]
enum BlackholeStatus {
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "stopped")]
    Stopped,
    #[serde(rename = "stopping")]
    Stopping,
}

#[derive(CandidType, Deserialize, Debug)]
struct BlackholeSettings {
    #[serde(default)]
    compute_allocation: Option<Nat>,
    controllers: Vec<Principal>,
    #[serde(default)]
    freezing_threshold: Option<Nat>,
    #[serde(default)]
    memory_allocation: Option<Nat>,
    // NNS Root extra settings (optional)
    #[serde(default)]
    wasm_memory_threshold: Option<Nat>,
    #[serde(default)]
    reserved_cycles_limit: Option<Nat>,
    #[serde(default)]
    log_visibility: Option<candid::Reserved>,
    #[serde(default)]
    wasm_memory_limit: Option<Nat>,
}

async fn query_canister_status(
    canister_id: Principal,
    proxy_id: Principal,
) -> CallResult<(BlackholeCanisterStatus,)> {
    let args = CanisterIdRecord { canister_id };
    ic_cdk::call(proxy_id, "canister_status", (args,)).await
}

// =============================================================================
// Query Functions
// =============================================================================

/// Get all tracked canisters (from hardcoded list)
#[ic_cdk::query]
fn get_canisters() -> Vec<CanisterInfo> {
    get_trackable_canisters()
        .into_iter()
        .map(|(canister_id, proxy_id)| CanisterInfo {
            canister_id,
            proxy_id,
        })
        .collect()
}

/// Get count of tracked canisters
#[ic_cdk::query]
fn get_canister_count() -> u64 {
    get_trackable_canisters().len() as u64
}

/// Get snapshots for a specific canister within a time range
#[ic_cdk::query]
fn get_snapshots(canister_id: Principal, from_ts: u64, to_ts: u64) -> Vec<Snapshot> {
    let start_key = SnapshotKey::new(canister_id, from_ts);
    let end_key = SnapshotKey::new(canister_id, to_ts);

    SNAPSHOTS.with(|s| {
        s.borrow()
            .range(start_key..=end_key)
            .map(|(k, v)| Snapshot {
                canister_id: k.to_principal(),
                timestamp: k.timestamp,
                cycles: v.0,
            })
            .collect()
    })
}

/// Get the latest snapshot for each canister
#[ic_cdk::query]
fn get_latest_snapshots() -> Vec<Snapshot> {
    let canisters = get_trackable_canisters();
    let mut results = Vec::new();

    SNAPSHOTS.with(|s| {
        let map = s.borrow();
        for (canister_id, _) in canisters {
            let end_key = SnapshotKey::new(canister_id, u64::MAX);
            let start_key = SnapshotKey::new(canister_id, 0);

            if let Some((k, v)) = map.range(start_key..=end_key).last() {
                results.push(Snapshot {
                    canister_id: k.to_principal(),
                    timestamp: k.timestamp,
                    cycles: v.0,
                });
            }
        }
    });

    results
}

/// Get total snapshot count
#[ic_cdk::query]
fn get_snapshot_count() -> u64 {
    SNAPSHOTS.with(|s| s.borrow().len())
}

/// Calculate burn rate for a canister based on recent snapshots
#[ic_cdk::query]
fn get_burn_rate(canister_id: Principal) -> Option<BurnRate> {
    let snapshots = get_snapshots(canister_id, 0, u64::MAX);

    if snapshots.len() < 2 {
        return None;
    }

    let mut sorted = snapshots;
    sorted.sort_by_key(|s| s.timestamp);

    let first = sorted.first().unwrap();
    let last = sorted.last().unwrap();

    let time_diff_nanos = last.timestamp.saturating_sub(first.timestamp);
    if time_diff_nanos == 0 {
        return None;
    }

    // Handle top-ups: if balance increased, treat as zero burn
    let cycles_burned = if first.cycles > last.cycles {
        first.cycles - last.cycles
    } else {
        0
    };

    let nanos_per_day: u128 = 24 * 60 * 60 * 1_000_000_000;
    let cycles_per_day = if time_diff_nanos > 0 {
        (cycles_burned * nanos_per_day) / (time_diff_nanos as u128)
    } else {
        0
    };

    let days_until_empty = if cycles_per_day > 0 {
        Some(last.cycles as f64 / cycles_per_day as f64)
    } else {
        None
    };

    Some(BurnRate {
        canister_id,
        cycles_per_day,
        latest_balance: last.cycles,
        days_until_empty,
        snapshots_used: sorted.len() as u64,
    })
}

/// Get burn rates for all canisters
#[ic_cdk::query]
fn get_all_burn_rates() -> Vec<BurnRate> {
    get_trackable_canisters()
        .into_iter()
        .filter_map(|(cid, _)| get_burn_rate(cid))
        .collect()
}

// =============================================================================
// Snapshot Collection (Public)
// =============================================================================

const BATCH_SIZE: usize = 30;

/// Take a snapshot of all tracked canisters. Public function.
#[ic_cdk::update]
async fn take_snapshot() -> SnapshotResult {
    let timestamp = now_nanos();
    let canisters = get_trackable_canisters();

    let total = canisters.len() as u64;
    let mut successful = 0u64;
    let mut failed = 0u64;

    for batch in canisters.chunks(BATCH_SIZE) {
        let futures: Vec<_> = batch
            .iter()
            .map(|(canister_id, proxy_id)| {
                let cid = *canister_id;
                let pid = *proxy_id;
                async move { (cid, query_canister_status(cid, pid).await) }
            })
            .collect();

        let results = futures::future::join_all(futures).await;

        for (canister_id, result) in results {
            match result {
                Ok((status,)) => {
                    let cycles = nat_to_u128(&status.cycles);
                    SNAPSHOTS.with(|s| {
                        s.borrow_mut().insert(
                            SnapshotKey::new(canister_id, timestamp),
                            CyclesValue(cycles),
                        );
                    });
                    successful += 1;
                }
                Err(e) => {
                    // Log first few errors for debugging
                    if failed < 3 {
                        ic_cdk::println!("Failed to query {}: {:?}", canister_id, e);
                    }
                    failed += 1;
                }
            }
        }
    }

    SnapshotResult {
        total_canisters: total,
        successful,
        failed,
        timestamp,
    }
}

ic_cdk::export_candid!();
