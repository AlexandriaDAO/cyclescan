use candid::{CandidType, Nat, Principal};
use ic_cdk::api::call::CallResult;
use ic_cdk_timers::TimerId;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Duration;

// =============================================================================
// Constants
// =============================================================================

const NANOS_PER_HOUR: u64 = 3_600_000_000_000;
const NANOS_PER_DAY: u64 = 86_400_000_000_000;
const SEVEN_DAYS_NANOS: u64 = 7 * NANOS_PER_DAY;
const BATCH_SIZE: usize = 50;
const SNAPSHOT_INTERVAL: Duration = Duration::from_secs(3600); // 1 hour

// =============================================================================
// Proxy Types - Extensible for future query methods
// =============================================================================

/// How to query a canister's cycles.
/// Extensible: add new variants for different query methods.
#[derive(CandidType, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProxyType {
    /// Query via blackhole's canister_status(canister_id)
    /// Used by: ninegua, NNS Root
    Blackhole,

    /// Query via SNS root's get_sns_canisters_summary()
    /// The proxy_id IS the SNS root, returns cycles for all SNS canisters
    SnsRoot,

    // Future examples:
    // OpenChat,      // If OpenChat exposes a custom status endpoint
    // Custom(String), // For arbitrary query methods
}

impl Default for ProxyType {
    fn default() -> Self {
        ProxyType::Blackhole
    }
}

// =============================================================================
// Types - API
// =============================================================================

/// Input for importing canisters
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CanisterImport {
    pub canister_id: Principal,
    pub proxy_id: Principal,
    #[serde(default)]
    pub proxy_type: ProxyType,
}

/// Leaderboard entry - the main output
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct LeaderboardEntry {
    pub canister_id: Principal,
    pub project: Option<String>,
    pub balance: u128,
    pub burn_1h: Option<u128>,
    pub burn_24h: Option<u128>,
    pub burn_7d: Option<u128>,
}

/// Result of take_snapshot
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct SnapshotResult {
    pub total: u64,
    pub success: u64,
    pub failed: u64,
    pub pruned: u64,
    pub timestamp: u64,
}

/// Canister stats
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct Stats {
    pub canister_count: u64,
    pub snapshot_count: u64,
    pub oldest_snapshot: Option<u64>,
    pub newest_snapshot: Option<u64>,
}

// =============================================================================
// Types - Storage
// =============================================================================

/// Canister metadata stored in stable memory
#[derive(Clone, Debug)]
struct CanisterMeta {
    proxy_id: Principal,
    proxy_type: ProxyType,
    project_name: Option<String>,
}

impl Storable for CanisterMeta {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let proxy_bytes = self.proxy_id.as_slice();
        let name_bytes = self.project_name.as_deref().unwrap_or("").as_bytes();
        let proxy_type_byte: u8 = match self.proxy_type {
            ProxyType::Blackhole => 0,
            ProxyType::SnsRoot => 1,
        };

        let mut bytes = Vec::with_capacity(2 + proxy_bytes.len() + 2 + name_bytes.len());
        bytes.push(proxy_bytes.len() as u8);
        bytes.extend_from_slice(proxy_bytes);
        bytes.push(proxy_type_byte);
        bytes.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        bytes.extend_from_slice(name_bytes);

        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let proxy_len = bytes[0] as usize;
        let proxy_id = Principal::from_slice(&bytes[1..1 + proxy_len]);

        let proxy_type_byte = bytes[1 + proxy_len];
        let proxy_type = match proxy_type_byte {
            1 => ProxyType::SnsRoot,
            _ => ProxyType::Blackhole,
        };

        let name_len_start = 2 + proxy_len;
        let name_len =
            u16::from_le_bytes([bytes[name_len_start], bytes[name_len_start + 1]]) as usize;

        let project_name = if name_len > 0 {
            let name_start = name_len_start + 2;
            Some(String::from_utf8_lossy(&bytes[name_start..name_start + name_len]).into_owned())
        } else {
            None
        };

        Self {
            proxy_id,
            proxy_type,
            project_name,
        }
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 1 + 29 + 1 + 2 + 100, // proxy len + proxy + type + name len + name
        is_fixed_size: false,
    };
}

/// Principal as storable key
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PrincipalKey([u8; 30]); // 1 byte len + 29 bytes principal

impl PrincipalKey {
    fn new(p: Principal) -> Self {
        let slice = p.as_slice();
        let mut bytes = [0u8; 30];
        bytes[0] = slice.len() as u8;
        bytes[1..1 + slice.len()].copy_from_slice(slice);
        Self(bytes)
    }

    fn to_principal(&self) -> Principal {
        let len = self.0[0] as usize;
        Principal::from_slice(&self.0[1..1 + len])
    }
}

impl Storable for PrincipalKey {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(&self.0)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let mut arr = [0u8; 30];
        arr.copy_from_slice(&bytes);
        Self(arr)
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 30,
        is_fixed_size: true,
    };
}

/// Snapshot key: (canister_id, timestamp)
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SnapshotKey {
    canister: PrincipalKey,
    timestamp: u64,
}

impl Storable for SnapshotKey {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let mut bytes = Vec::with_capacity(38);
        bytes.extend_from_slice(&self.canister.0);
        bytes.extend_from_slice(&self.timestamp.to_be_bytes());
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let mut canister_bytes = [0u8; 30];
        canister_bytes.copy_from_slice(&bytes[0..30]);
        let timestamp = u64::from_be_bytes(bytes[30..38].try_into().unwrap());
        Self {
            canister: PrincipalKey(canister_bytes),
            timestamp,
        }
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: 38,
        is_fixed_size: true,
    };
}

/// Cycles value
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

const CANISTERS_MEM_ID: MemoryId = MemoryId::new(0);
const SNAPSHOTS_MEM_ID: MemoryId = MemoryId::new(1);

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static CANISTERS: RefCell<StableBTreeMap<PrincipalKey, CanisterMeta, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(CANISTERS_MEM_ID))
        ));

    static SNAPSHOTS: RefCell<StableBTreeMap<SnapshotKey, CyclesValue, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(SNAPSHOTS_MEM_ID))
        ));

    static SNAPSHOT_TIMER_ID: RefCell<Option<TimerId>> = RefCell::new(None);
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

fn is_controller() -> bool {
    let caller = ic_cdk::caller();
    ic_cdk::api::is_controller(&caller)
}

/// Calculate burn for a time window. Returns None if insufficient data.
/// Treats top-ups (cycles increase) as zero burn.
fn calculate_burn(canister: &PrincipalKey, window_nanos: u64, now: u64) -> Option<u128> {
    let cutoff = now.saturating_sub(window_nanos);

    SNAPSHOTS.with(|s| {
        let map = s.borrow();

        let start_key = SnapshotKey {
            canister: canister.clone(),
            timestamp: cutoff,
        };
        let end_key = SnapshotKey {
            canister: canister.clone(),
            timestamp: now,
        };

        let snapshots: Vec<_> = map.range(start_key..=end_key).collect();

        if snapshots.len() < 2 {
            return None;
        }

        let earliest_cycles = snapshots.first().unwrap().1 .0;
        let latest_cycles = snapshots.last().unwrap().1 .0;

        if latest_cycles >= earliest_cycles {
            Some(0)
        } else {
            Some(earliest_cycles - latest_cycles)
        }
    })
}

// =============================================================================
// Blackhole Query (ninegua, NNS Root)
// =============================================================================

#[derive(CandidType, Deserialize)]
struct CanisterIdRecord {
    canister_id: Principal,
}

#[derive(CandidType, Deserialize, Debug)]
struct BlackholeStatus {
    #[allow(dead_code)]
    status: BlackholeRunningStatus,
    #[allow(dead_code)]
    memory_size: Nat,
    cycles: Nat,
    #[allow(dead_code)]
    settings: BlackholeSettings,
    #[serde(default)]
    #[allow(dead_code)]
    module_hash: Option<Vec<u8>>,
    #[serde(default)]
    #[allow(dead_code)]
    idle_cycles_burned_per_day: Option<Nat>,
    #[serde(default)]
    #[allow(dead_code)]
    reserved_cycles: Option<Nat>,
}

#[derive(CandidType, Deserialize, Debug)]
enum BlackholeRunningStatus {
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "stopped")]
    Stopped,
    #[serde(rename = "stopping")]
    Stopping,
}

#[derive(CandidType, Deserialize, Debug)]
struct BlackholeSettings {
    #[allow(dead_code)]
    controllers: Vec<Principal>,
    #[serde(default)]
    #[allow(dead_code)]
    compute_allocation: Option<Nat>,
    #[serde(default)]
    #[allow(dead_code)]
    freezing_threshold: Option<Nat>,
    #[serde(default)]
    #[allow(dead_code)]
    memory_allocation: Option<Nat>,
}

async fn query_blackhole(canister_id: Principal, proxy_id: Principal) -> CallResult<u128> {
    let args = CanisterIdRecord { canister_id };
    let result: CallResult<(BlackholeStatus,)> =
        ic_cdk::call(proxy_id, "canister_status", (args,)).await;
    result.map(|(status,)| nat_to_u128(&status.cycles))
}

// =============================================================================
// SNS Root Query
// =============================================================================

#[derive(CandidType, Deserialize, Debug)]
struct SnsCanistersSummary {
    root: Option<SnsCanisterSummary>,
    governance: Option<SnsCanisterSummary>,
    ledger: Option<SnsCanisterSummary>,
    index: Option<SnsCanisterSummary>,
    swap: Option<SnsCanisterSummary>,
    #[serde(default)]
    dapps: Vec<SnsCanisterSummary>,
    #[serde(default)]
    archives: Vec<SnsCanisterSummary>,
}

#[derive(CandidType, Deserialize, Debug)]
struct SnsCanisterSummary {
    canister_id: Option<Principal>,
    status: Option<SnsCanisterStatus>,
}

#[derive(CandidType, Deserialize, Debug)]
struct SnsCanisterStatus {
    cycles: Nat,
    #[serde(default)]
    #[allow(dead_code)]
    memory_size: Nat,
    #[serde(default)]
    #[allow(dead_code)]
    status: Option<SnsRunningStatus>,
}

#[derive(CandidType, Deserialize, Debug)]
enum SnsRunningStatus {
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "stopped")]
    Stopped,
    #[serde(rename = "stopping")]
    Stopping,
}

#[derive(CandidType, Deserialize)]
struct EmptyRecord {}

/// Query SNS root for all canister cycles. Returns map of canister_id -> cycles
async fn query_sns_root(sns_root: Principal) -> CallResult<Vec<(Principal, u128)>> {
    let result: CallResult<(SnsCanistersSummary,)> =
        ic_cdk::call(sns_root, "get_sns_canisters_summary", (EmptyRecord {},)).await;

    result.map(|(summary,)| {
        let mut results = Vec::new();

        // Helper to extract canister_id and cycles
        let extract = |opt: Option<&SnsCanisterSummary>| -> Option<(Principal, u128)> {
            let s = opt?;
            let id = s.canister_id?;
            let status = s.status.as_ref()?;
            Some((id, nat_to_u128(&status.cycles)))
        };

        if let Some(entry) = extract(summary.root.as_ref()) {
            results.push(entry);
        }
        if let Some(entry) = extract(summary.governance.as_ref()) {
            results.push(entry);
        }
        if let Some(entry) = extract(summary.ledger.as_ref()) {
            results.push(entry);
        }
        if let Some(entry) = extract(summary.index.as_ref()) {
            results.push(entry);
        }
        if let Some(entry) = extract(summary.swap.as_ref()) {
            results.push(entry);
        }
        for dapp in &summary.dapps {
            if let Some(entry) = extract(Some(dapp)) {
                results.push(entry);
            }
        }
        for archive in &summary.archives {
            if let Some(entry) = extract(Some(archive)) {
                results.push(entry);
            }
        }

        results
    })
}

// =============================================================================
// Query Functions
// =============================================================================

/// Get the leaderboard - main query
#[ic_cdk::query]
fn get_leaderboard() -> Vec<LeaderboardEntry> {
    let now = now_nanos();

    CANISTERS.with(|c| {
        let canisters = c.borrow();
        let mut entries: Vec<LeaderboardEntry> = canisters
            .iter()
            .map(|(key, meta)| {
                let canister_id = key.to_principal();

                let balance = SNAPSHOTS.with(|s| {
                    let map = s.borrow();
                    let end_key = SnapshotKey {
                        canister: key.clone(),
                        timestamp: u64::MAX,
                    };
                    let start_key = SnapshotKey {
                        canister: key.clone(),
                        timestamp: 0,
                    };
                    map.range(start_key..=end_key)
                        .last()
                        .map(|(_, v)| v.0)
                        .unwrap_or(0)
                });

                LeaderboardEntry {
                    canister_id,
                    project: meta.project_name.clone(),
                    balance,
                    burn_1h: calculate_burn(&key, NANOS_PER_HOUR, now),
                    burn_24h: calculate_burn(&key, NANOS_PER_DAY, now),
                    burn_7d: calculate_burn(&key, SEVEN_DAYS_NANOS, now),
                }
            })
            .collect();

        entries.sort_by(|a, b| {
            let a_burn = a.burn_24h.unwrap_or(0);
            let b_burn = b.burn_24h.unwrap_or(0);
            b_burn.cmp(&a_burn)
        });

        entries
    })
}

/// Get stats
#[ic_cdk::query]
fn get_stats() -> Stats {
    let canister_count = CANISTERS.with(|c| c.borrow().len());
    let (snapshot_count, oldest, newest) = SNAPSHOTS.with(|s| {
        let map = s.borrow();
        let count = map.len();
        let oldest = map.first_key_value().map(|(k, _)| k.timestamp);
        let newest = map.last_key_value().map(|(k, _)| k.timestamp);
        (count, oldest, newest)
    });

    Stats {
        canister_count,
        snapshot_count,
        oldest_snapshot: oldest,
        newest_snapshot: newest,
    }
}

/// Get canister count
#[ic_cdk::query]
fn get_canister_count() -> u64 {
    CANISTERS.with(|c| c.borrow().len())
}

// =============================================================================
// Update Functions
// =============================================================================

/// Import canisters (controller only)
#[ic_cdk::update]
fn import_canisters(canisters: Vec<CanisterImport>) -> u64 {
    if !is_controller() {
        ic_cdk::trap("Only controller can import canisters");
    }

    let mut count = 0u64;
    CANISTERS.with(|c| {
        let mut map = c.borrow_mut();
        for import in canisters {
            let key = PrincipalKey::new(import.canister_id);
            let existing_name = map.get(&key).and_then(|m| m.project_name.clone());
            map.insert(
                key,
                CanisterMeta {
                    proxy_id: import.proxy_id,
                    proxy_type: import.proxy_type,
                    project_name: existing_name,
                },
            );
            count += 1;
        }
    });
    count
}

/// Set project name for a canister (controller only)
#[ic_cdk::update]
fn set_project(canister_id: Principal, project: Option<String>) {
    if !is_controller() {
        ic_cdk::trap("Only controller can set project names");
    }

    let key = PrincipalKey::new(canister_id);
    CANISTERS.with(|c| {
        let mut map = c.borrow_mut();
        if let Some(mut meta) = map.get(&key) {
            meta.project_name = project;
            map.insert(key, meta);
        }
    });
}

/// Take a snapshot of all canisters
#[ic_cdk::update]
async fn take_snapshot() -> SnapshotResult {
    let timestamp = now_nanos();

    // Collect all canisters grouped by proxy type
    let (blackhole_canisters, sns_roots): (Vec<_>, Vec<_>) = CANISTERS.with(|c| {
        let map = c.borrow();
        let mut blackhole = Vec::new();
        let mut sns: HashMap<Principal, Vec<Principal>> = HashMap::new();

        for (key, meta) in map.iter() {
            let canister_id = key.to_principal();
            match meta.proxy_type {
                ProxyType::Blackhole => {
                    blackhole.push((canister_id, meta.proxy_id));
                }
                ProxyType::SnsRoot => {
                    // Group by SNS root
                    sns.entry(meta.proxy_id)
                        .or_default()
                        .push(canister_id);
                }
            }
        }

        (blackhole, sns.into_iter().collect())
    });

    let total = CANISTERS.with(|c| c.borrow().len()) as u64;
    let mut success = 0u64;
    let mut failed = 0u64;

    // Process blackhole canisters in batches
    for batch in blackhole_canisters.chunks(BATCH_SIZE) {
        let futures: Vec<_> = batch
            .iter()
            .map(|(canister_id, proxy_id)| {
                let cid = *canister_id;
                let pid = *proxy_id;
                async move { (cid, query_blackhole(cid, pid).await) }
            })
            .collect();

        let results = futures::future::join_all(futures).await;

        for (canister_id, result) in results {
            match result {
                Ok(cycles) => {
                    SNAPSHOTS.with(|s| {
                        s.borrow_mut().insert(
                            SnapshotKey {
                                canister: PrincipalKey::new(canister_id),
                                timestamp,
                            },
                            CyclesValue(cycles),
                        );
                    });
                    success += 1;
                }
                Err(e) => {
                    if failed < 5 {
                        ic_cdk::println!("Blackhole failed {}: {:?}", canister_id, e);
                    }
                    failed += 1;
                }
            }
        }
    }

    // Process SNS roots (one call per root returns all canisters)
    for batch in sns_roots.chunks(BATCH_SIZE) {
        let futures: Vec<_> = batch
            .iter()
            .map(|(sns_root, expected_canisters)| {
                let root = *sns_root;
                let expected = expected_canisters.clone();
                async move { (root, expected, query_sns_root(root).await) }
            })
            .collect();

        let results = futures::future::join_all(futures).await;

        for (sns_root, expected_canisters, result) in results {
            match result {
                Ok(canister_cycles) => {
                    // Only record cycles for canisters we're tracking
                    let expected_set: std::collections::HashSet<_> =
                        expected_canisters.iter().collect();

                    for (canister_id, cycles) in canister_cycles {
                        if expected_set.contains(&canister_id) {
                            SNAPSHOTS.with(|s| {
                                s.borrow_mut().insert(
                                    SnapshotKey {
                                        canister: PrincipalKey::new(canister_id),
                                        timestamp,
                                    },
                                    CyclesValue(cycles),
                                );
                            });
                            success += 1;
                        }
                    }
                }
                Err(e) => {
                    if failed < 5 {
                        ic_cdk::println!("SNS root failed {}: {:?}", sns_root, e);
                    }
                    failed += expected_canisters.len() as u64;
                }
            }
        }
    }

    // Prune old snapshots
    let cutoff = timestamp.saturating_sub(SEVEN_DAYS_NANOS);
    let pruned = SNAPSHOTS.with(|s| {
        let mut map = s.borrow_mut();
        let mut to_remove = Vec::new();

        for (key, _) in map.iter() {
            if key.timestamp < cutoff {
                to_remove.push(key.clone());
            } else {
                break;
            }
        }

        let count = to_remove.len() as u64;
        for key in to_remove {
            map.remove(&key);
        }
        count
    });

    SnapshotResult {
        total,
        success,
        failed,
        pruned,
        timestamp,
    }
}

/// Clear all canisters (controller only)
#[ic_cdk::update]
fn clear_canisters() {
    if !is_controller() {
        ic_cdk::trap("Only controller can clear canisters");
    }

    CANISTERS.with(|c| {
        let mut map = c.borrow_mut();
        let keys: Vec<_> = map.iter().map(|(k, _)| k.clone()).collect();
        for key in keys {
            map.remove(&key);
        }
    });
}

/// Clear all snapshots (controller only)
#[ic_cdk::update]
fn clear_snapshots() {
    if !is_controller() {
        ic_cdk::trap("Only controller can clear snapshots");
    }

    SNAPSHOTS.with(|s| {
        let mut map = s.borrow_mut();
        let keys: Vec<_> = map.iter().map(|(k, _)| k.clone()).collect();
        for key in keys {
            map.remove(&key);
        }
    });
}

// =============================================================================
// Timer Functions
// =============================================================================

fn schedule_snapshot_timer() {
    let timer_id = ic_cdk_timers::set_timer_interval(SNAPSHOT_INTERVAL, || {
        ic_cdk::spawn(async {
            let result = take_snapshot().await;
            ic_cdk::println!(
                "Auto snapshot: {} success, {} failed",
                result.success,
                result.failed
            );
        });
    });

    SNAPSHOT_TIMER_ID.with(|id| {
        *id.borrow_mut() = Some(timer_id);
    });

    ic_cdk::println!("Snapshot timer scheduled (hourly)");
}

/// Start the automatic snapshot timer (controller only)
#[ic_cdk::update]
fn start_timer() {
    if !is_controller() {
        ic_cdk::trap("Only controller can start timer");
    }

    // Cancel existing timer if any
    SNAPSHOT_TIMER_ID.with(|id| {
        if let Some(timer_id) = id.borrow_mut().take() {
            ic_cdk_timers::clear_timer(timer_id);
        }
    });

    schedule_snapshot_timer();
}

/// Stop the automatic snapshot timer (controller only)
#[ic_cdk::update]
fn stop_timer() {
    if !is_controller() {
        ic_cdk::trap("Only controller can stop timer");
    }

    SNAPSHOT_TIMER_ID.with(|id| {
        if let Some(timer_id) = id.borrow_mut().take() {
            ic_cdk_timers::clear_timer(timer_id);
            ic_cdk::println!("Snapshot timer stopped");
        }
    });
}

/// Check if timer is running
#[ic_cdk::query]
fn is_timer_running() -> bool {
    SNAPSHOT_TIMER_ID.with(|id| id.borrow().is_some())
}

// =============================================================================
// Lifecycle Hooks
// =============================================================================

#[ic_cdk::init]
fn init() {
    schedule_snapshot_timer();
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    schedule_snapshot_timer();
}

ic_cdk::export_candid!();
