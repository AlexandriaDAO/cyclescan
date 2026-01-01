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
const THIRTY_DAYS_NANOS: u64 = 30 * NANOS_PER_DAY;
const RETENTION_PERIOD: u64 = THIRTY_DAYS_NANOS;
const BATCH_SIZE: usize = 50;
const SNAPSHOT_INTERVAL: Duration = Duration::from_secs(3600); // 1 hour
const MAX_PAGE_LIMIT: u64 = 1000;
const MAX_PROJECT_NAME_BYTES: usize = 100;
const MAX_WEBSITE_BYTES: usize = 200;

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

/// Input for importing canisters (also used for export round-trip)
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct CanisterImport {
    pub canister_id: Principal,
    pub proxy_id: Principal,
    #[serde(default)]
    pub proxy_type: ProxyType,
    #[serde(default)]
    pub project: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
}

/// Full canister info for queries
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct CanisterInfo {
    pub canister_id: Principal,
    pub proxy_id: Principal,
    pub proxy_type: ProxyType,
    pub project: Option<String>,
    pub website: Option<String>,
}

/// Partial update for canister metadata
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CanisterUpdate {
    pub proxy_id: Option<Principal>,
    pub proxy_type: Option<ProxyType>,
    /// None = unchanged, Some(None) = clear, Some(Some(x)) = set to x
    pub project: Option<Option<String>>,
    /// None = unchanged, Some(None) = clear, Some(Some(x)) = set to x
    pub website: Option<Option<String>>,
}

/// Leaderboard entry - the main output
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct LeaderboardEntry {
    pub canister_id: Principal,
    pub project: Option<String>,
    pub website: Option<String>,
    pub balance: u128,
    pub burn_1h: Option<u128>,
    pub burn_24h: Option<u128>,
    pub burn_7d: Option<u128>,
}

/// Project-aggregated leaderboard entry
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct ProjectLeaderboardEntry {
    pub project: String,
    pub website: Option<String>,
    pub canister_count: u64,
    pub total_balance: u128,
    pub total_burn_1h: Option<u128>,
    pub total_burn_24h: Option<u128>,
    pub total_burn_7d: Option<u128>,
}

/// Paginated leaderboard response
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct LeaderboardPage {
    pub entries: Vec<LeaderboardEntry>,
    pub total: u64,
    pub offset: u64,
    pub limit: u64,
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

/// A single snapshot point for history (used by detail modal)
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct SnapshotPoint {
    pub timestamp: u64,
    pub cycles: u128,
}

/// Full history for a canister (for detail view modal)
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct CanisterHistory {
    pub canister_id: Principal,
    pub project: Option<String>,
    pub website: Option<String>,
    pub current_balance: u128,
    pub snapshots: Vec<SnapshotPoint>,
    pub burn_1h: Option<u128>,
    pub burn_24h: Option<u128>,
    pub burn_7d: Option<u128>,
    pub burn_30d: Option<u128>,
    pub is_24h_actual: bool,
    pub is_7d_actual: bool,
    pub is_30d_actual: bool,
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
    website: Option<String>,
}

/// Schema version for CanisterMeta serialization.
/// Version detection: if first byte < 128, it's v0 (legacy). If >= 128, version = byte - 128.
/// v1: Added version marker
/// v2: Added website field
const CANISTER_META_VERSION: u8 = 2;
const VERSION_MARKER: u8 = 128; // High bit set to distinguish from v0's proxy_len (0-29)

impl Storable for CanisterMeta {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let proxy_bytes = self.proxy_id.as_slice();
        let name_bytes = self.project_name.as_deref().unwrap_or("").as_bytes();
        let website_bytes = self.website.as_deref().unwrap_or("").as_bytes();
        let proxy_type_byte: u8 = match self.proxy_type {
            ProxyType::Blackhole => 0,
            ProxyType::SnsRoot => 1,
        };

        // v2 format: [version | proxy_len | proxy | proxy_type | name_len (u16 LE) | name | website_len (u16 LE) | website]
        let mut bytes = Vec::with_capacity(1 + 1 + proxy_bytes.len() + 1 + 2 + name_bytes.len() + 2 + website_bytes.len());
        bytes.push(VERSION_MARKER + CANISTER_META_VERSION); // Version byte (130 = v2)
        bytes.push(proxy_bytes.len() as u8);
        bytes.extend_from_slice(proxy_bytes);
        bytes.push(proxy_type_byte);
        bytes.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        bytes.extend_from_slice(name_bytes);
        bytes.extend_from_slice(&(website_bytes.len() as u16).to_le_bytes());
        bytes.extend_from_slice(website_bytes);

        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let first_byte = bytes[0];

        // Detect version: v0 has proxy_len (0-29) as first byte, v1+ has 128+ as version marker
        if first_byte < VERSION_MARKER {
            // v0 (legacy format): [proxy_len | proxy | proxy_type | name_len (u16 LE) | name]
            let proxy_len = first_byte as usize;
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
                website: None, // v0 has no website
            }
        } else {
            let version = first_byte - VERSION_MARKER;
            let proxy_len = bytes[1] as usize;
            let proxy_id = Principal::from_slice(&bytes[2..2 + proxy_len]);

            let proxy_type_byte = bytes[2 + proxy_len];
            let proxy_type = match proxy_type_byte {
                1 => ProxyType::SnsRoot,
                _ => ProxyType::Blackhole,
            };

            let name_len_start = 3 + proxy_len;
            let name_len =
                u16::from_le_bytes([bytes[name_len_start], bytes[name_len_start + 1]]) as usize;

            let project_name = if name_len > 0 {
                let name_start = name_len_start + 2;
                Some(String::from_utf8_lossy(&bytes[name_start..name_start + name_len]).into_owned())
            } else {
                None
            };

            // v2+ has website field
            let website = if version >= 2 {
                let website_len_start = name_len_start + 2 + name_len;
                let website_len =
                    u16::from_le_bytes([bytes[website_len_start], bytes[website_len_start + 1]]) as usize;

                if website_len > 0 {
                    let website_start = website_len_start + 2;
                    Some(String::from_utf8_lossy(&bytes[website_start..website_start + website_len]).into_owned())
                } else {
                    None
                }
            } else {
                None // v1 has no website
            };

            Self {
                proxy_id,
                proxy_type,
                project_name,
                website,
            }
        }
    }

    const BOUND: Bound = Bound::Bounded {
        // version + proxy_len + proxy + type + name_len + name + website_len + website
        max_size: 1 + 1 + 29 + 1 + 2 + 100 + 2 + 200,
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

/// Truncate project name to max allowed bytes (UTF-8 safe)
fn sanitize_project_name(name: Option<String>) -> Option<String> {
    name.map(|s| {
        if s.len() <= MAX_PROJECT_NAME_BYTES {
            s
        } else {
            // Truncate at valid UTF-8 boundary
            let mut end = MAX_PROJECT_NAME_BYTES;
            while end > 0 && !s.is_char_boundary(end) {
                end -= 1;
            }
            s[..end].to_string()
        }
    })
}

/// Truncate website URL to max allowed bytes (UTF-8 safe)
fn sanitize_website(url: Option<String>) -> Option<String> {
    url.map(|s| {
        if s.len() <= MAX_WEBSITE_BYTES {
            s
        } else {
            // Truncate at valid UTF-8 boundary
            let mut end = MAX_WEBSITE_BYTES;
            while end > 0 && !s.is_char_boundary(end) {
                end -= 1;
            }
            s[..end].to_string()
        }
    })
}

/// Delete all snapshots for a canister (used for cascade delete)
fn delete_snapshots_for_canister(canister: &PrincipalKey) -> u64 {
    SNAPSHOTS.with(|s| {
        let mut map = s.borrow_mut();

        let start_key = SnapshotKey {
            canister: canister.clone(),
            timestamp: 0,
        };
        let end_key = SnapshotKey {
            canister: canister.clone(),
            timestamp: u64::MAX,
        };

        let keys_to_remove: Vec<_> = map
            .range(start_key..=end_key)
            .map(|(k, _)| k.clone())
            .collect();

        let count = keys_to_remove.len() as u64;
        for key in keys_to_remove {
            map.remove(&key);
        }
        count
    })
}

/// Calculate burn for a time window using hybrid approach:
/// - If we have actual data spanning the window, sum burns between consecutive snapshots
///   (ignoring top-ups, which would show as increases)
/// - Otherwise, extrapolate from the last 2 snapshots
fn calculate_burn(canister: &PrincipalKey, window_nanos: u64, now: u64) -> Option<u128> {
    SNAPSHOTS.with(|s| {
        let map = s.borrow();

        // Get all snapshots for this canister
        let start_key = SnapshotKey {
            canister: canister.clone(),
            timestamp: 0,
        };
        let end_key = SnapshotKey {
            canister: canister.clone(),
            timestamp: u64::MAX,
        };

        let snapshots: Vec<_> = map.range(start_key..=end_key).collect();

        if snapshots.len() < 2 {
            return None;
        }

        // Calculate cutoff for the requested window
        let cutoff = now.saturating_sub(window_nanos);

        // Find the first snapshot at or after the cutoff
        let start_idx = snapshots
            .iter()
            .position(|(key, _)| key.timestamp >= cutoff);

        match start_idx {
            Some(idx) if idx < snapshots.len() - 1 => {
                // ACTUAL DATA: Sum burns between consecutive snapshots in the window
                // This correctly handles top-ups by only counting decreases
                let mut total_burn: u128 = 0;

                for i in idx..snapshots.len() - 1 {
                    let older_cycles = snapshots[i].1 .0;
                    let newer_cycles = snapshots[i + 1].1 .0;

                    // Only count burns (decreases), not top-ups (increases)
                    if older_cycles > newer_cycles {
                        total_burn += older_cycles - newer_cycles;
                    }
                }

                Some(total_burn)
            }
            _ => {
                // EXTRAPOLATE: Not enough historical data, use last 2 snapshots
                let len = snapshots.len();
                let (older_key, older_val) = &snapshots[len - 2];
                let (newer_key, newer_val) = &snapshots[len - 1];

                let older_cycles = older_val.0;
                let newer_cycles = newer_val.0;
                let time_elapsed = newer_key.timestamp.saturating_sub(older_key.timestamp);

                if time_elapsed == 0 {
                    return Some(0);
                }

                // Handle top-ups
                if newer_cycles >= older_cycles {
                    return Some(0);
                }

                let actual_burn = older_cycles - newer_cycles;

                // Extrapolate to requested window
                let burn_per_nano = actual_burn as f64 / time_elapsed as f64;
                let projected_burn = burn_per_nano * window_nanos as f64;

                Some(projected_burn as u128)
            }
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
                    website: meta.website.clone(),
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

/// Get paginated leaderboard with offset and limit
#[ic_cdk::query]
fn get_leaderboard_page(offset: u64, limit: u64) -> LeaderboardPage {
    let now = now_nanos();
    let limit = limit.min(MAX_PAGE_LIMIT);

    CANISTERS.with(|c| {
        let canisters = c.borrow();
        let total = canisters.len();

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
                    website: meta.website.clone(),
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

        let page_entries: Vec<LeaderboardEntry> = entries
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect();

        LeaderboardPage {
            entries: page_entries,
            total,
            offset,
            limit,
        }
    })
}

/// Get the project-aggregated leaderboard (excludes canisters without a project)
#[ic_cdk::query]
fn get_project_leaderboard() -> Vec<ProjectLeaderboardEntry> {
    let now = now_nanos();

    // First, get individual canister data (including website)
    let canister_data: Vec<(PrincipalKey, String, Option<String>, u128, Option<u128>, Option<u128>, Option<u128>)> =
        CANISTERS.with(|c| {
            let canisters = c.borrow();
            canisters
                .iter()
                .filter_map(|(key, meta)| {
                    // Only include canisters with a project name
                    let project_name = meta.project_name.as_ref()?;

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

                    Some((
                        key.clone(),
                        project_name.clone(),
                        meta.website.clone(),
                        balance,
                        calculate_burn(&key, NANOS_PER_HOUR, now),
                        calculate_burn(&key, NANOS_PER_DAY, now),
                        calculate_burn(&key, SEVEN_DAYS_NANOS, now),
                    ))
                })
                .collect()
        });

    // Aggregate by project
    let mut project_map: HashMap<String, ProjectLeaderboardEntry> = HashMap::new();

    for (_key, project, website, balance, burn_1h, burn_24h, burn_7d) in canister_data {
        let entry = project_map.entry(project.clone()).or_insert(ProjectLeaderboardEntry {
            project,
            website: None,
            canister_count: 0,
            total_balance: 0,
            total_burn_1h: Some(0),
            total_burn_24h: Some(0),
            total_burn_7d: Some(0),
        });

        // Use the first non-None website we encounter
        if entry.website.is_none() && website.is_some() {
            entry.website = website;
        }

        entry.canister_count += 1;
        entry.total_balance += balance;

        // Sum burns (treating None as 0 for aggregation)
        if let Some(b) = burn_1h {
            *entry.total_burn_1h.as_mut().unwrap() += b;
        }
        if let Some(b) = burn_24h {
            *entry.total_burn_24h.as_mut().unwrap() += b;
        }
        if let Some(b) = burn_7d {
            *entry.total_burn_7d.as_mut().unwrap() += b;
        }
    }

    // Convert to vec and sort by 24h burn descending
    let mut entries: Vec<ProjectLeaderboardEntry> = project_map.into_values().collect();
    entries.sort_by(|a, b| {
        let a_burn = a.total_burn_24h.unwrap_or(0);
        let b_burn = b.total_burn_24h.unwrap_or(0);
        b_burn.cmp(&a_burn)
    });

    entries
}

/// Get all canisters for a specific project
#[ic_cdk::query]
fn get_project_canisters(project_name: String) -> Vec<LeaderboardEntry> {
    let now = now_nanos();

    CANISTERS.with(|c| {
        let canisters = c.borrow();
        let mut entries: Vec<LeaderboardEntry> = canisters
            .iter()
            .filter_map(|(key, meta)| {
                // Only include canisters with matching project name
                let canister_project = meta.project_name.as_ref()?;
                if canister_project != &project_name {
                    return None;
                }

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

                Some(LeaderboardEntry {
                    canister_id,
                    project: Some(project_name.clone()),
                    website: meta.website.clone(),
                    balance,
                    burn_1h: calculate_burn(&key, NANOS_PER_HOUR, now),
                    burn_24h: calculate_burn(&key, NANOS_PER_DAY, now),
                    burn_7d: calculate_burn(&key, SEVEN_DAYS_NANOS, now),
                })
            })
            .collect();

        // Sort by 24h burn descending
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

/// Get full details for a single canister
#[ic_cdk::query]
fn get_canister(canister_id: Principal) -> Option<CanisterInfo> {
    let key = PrincipalKey::new(canister_id);
    CANISTERS.with(|c| {
        c.borrow().get(&key).map(|meta| CanisterInfo {
            canister_id,
            proxy_id: meta.proxy_id,
            proxy_type: meta.proxy_type,
            project: meta.project_name,
            website: meta.website,
        })
    })
}

/// List all canisters with full details
#[ic_cdk::query]
fn list_canisters() -> Vec<CanisterInfo> {
    CANISTERS.with(|c| {
        c.borrow()
            .iter()
            .map(|(key, meta)| CanisterInfo {
                canister_id: key.to_principal(),
                proxy_id: meta.proxy_id,
                proxy_type: meta.proxy_type,
                project: meta.project_name,
                website: meta.website,
            })
            .collect()
    })
}

/// Export all canisters in import-compatible format (for backup/round-trip)
#[ic_cdk::query]
fn export_canisters() -> Vec<CanisterImport> {
    CANISTERS.with(|c| {
        c.borrow()
            .iter()
            .map(|(key, meta)| CanisterImport {
                canister_id: key.to_principal(),
                proxy_id: meta.proxy_id,
                proxy_type: meta.proxy_type,
                project: meta.project_name,
                website: meta.website,
            })
            .collect()
    })
}

/// Get full history for a specific canister (for detail modal)
#[ic_cdk::query]
fn get_canister_history(canister_id: Principal) -> Option<CanisterHistory> {
    let key = PrincipalKey::new(canister_id);
    let now = now_nanos();

    // Get canister metadata
    let meta = CANISTERS.with(|c| c.borrow().get(&key))?;

    // Get all snapshots for this canister
    let snapshots: Vec<SnapshotPoint> = SNAPSHOTS.with(|s| {
        let map = s.borrow();

        let start_key = SnapshotKey {
            canister: key.clone(),
            timestamp: 0,
        };
        let end_key = SnapshotKey {
            canister: key.clone(),
            timestamp: u64::MAX,
        };

        map.range(start_key..=end_key)
            .map(|(k, v)| SnapshotPoint {
                timestamp: k.timestamp,
                cycles: v.0,
            })
            .collect()
    });

    if snapshots.is_empty() {
        return None;
    }

    let current_balance = snapshots.last().map(|s| s.cycles).unwrap_or(0);

    // Calculate burns
    let burn_1h = calculate_burn(&key, NANOS_PER_HOUR, now);
    let burn_24h = calculate_burn(&key, NANOS_PER_DAY, now);
    let burn_7d = calculate_burn(&key, SEVEN_DAYS_NANOS, now);
    let burn_30d = calculate_burn(&key, THIRTY_DAYS_NANOS, now);

    // Determine if we have actual data for each window
    let oldest_timestamp = snapshots.first().map(|s| s.timestamp).unwrap_or(now);
    let data_span = now.saturating_sub(oldest_timestamp);

    let is_24h_actual = data_span >= NANOS_PER_DAY;
    let is_7d_actual = data_span >= SEVEN_DAYS_NANOS;
    let is_30d_actual = data_span >= THIRTY_DAYS_NANOS;

    Some(CanisterHistory {
        canister_id,
        project: meta.project_name,
        website: meta.website,
        current_balance,
        snapshots,
        burn_1h,
        burn_24h,
        burn_7d,
        burn_30d,
        is_24h_actual,
        is_7d_actual,
        is_30d_actual,
    })
}

// =============================================================================
// Update Functions
// =============================================================================

/// Import canisters (controller only)
/// If project/website is provided in import, it will be used; otherwise preserves existing values
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
            let existing = map.get(&key);
            // Use provided values, or fall back to existing ones
            // Sanitize to ensure they fit in storage
            let project_name = sanitize_project_name(
                import
                    .project
                    .or_else(|| existing.as_ref().and_then(|m| m.project_name.clone())),
            );
            let website = sanitize_website(
                import
                    .website
                    .or_else(|| existing.as_ref().and_then(|m| m.website.clone())),
            );
            map.insert(
                key,
                CanisterMeta {
                    proxy_id: import.proxy_id,
                    proxy_type: import.proxy_type,
                    project_name,
                    website,
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
            meta.project_name = sanitize_project_name(project);
            map.insert(key, meta);
        }
    });
}

/// Set project names in bulk (controller only)
#[ic_cdk::update]
fn set_projects(projects: Vec<(Principal, Option<String>)>) -> u64 {
    if !is_controller() {
        ic_cdk::trap("Only controller can set project names");
    }

    let mut count = 0u64;
    CANISTERS.with(|c| {
        let mut map = c.borrow_mut();
        for (canister_id, project) in projects {
            let key = PrincipalKey::new(canister_id);
            if let Some(mut meta) = map.get(&key) {
                meta.project_name = sanitize_project_name(project);
                map.insert(key, meta);
                count += 1;
            }
        }
    });
    count
}

/// Set website URL for a canister (controller only)
#[ic_cdk::update]
fn set_website(canister_id: Principal, website: Option<String>) {
    if !is_controller() {
        ic_cdk::trap("Only controller can set website URLs");
    }

    let key = PrincipalKey::new(canister_id);
    CANISTERS.with(|c| {
        let mut map = c.borrow_mut();
        if let Some(mut meta) = map.get(&key) {
            meta.website = sanitize_website(website);
            map.insert(key, meta);
        }
    });
}

/// Set website URLs in bulk (controller only)
#[ic_cdk::update]
fn set_websites(websites: Vec<(Principal, Option<String>)>) -> u64 {
    if !is_controller() {
        ic_cdk::trap("Only controller can set website URLs");
    }

    let mut count = 0u64;
    CANISTERS.with(|c| {
        let mut map = c.borrow_mut();
        for (canister_id, website) in websites {
            let key = PrincipalKey::new(canister_id);
            if let Some(mut meta) = map.get(&key) {
                meta.website = sanitize_website(website);
                map.insert(key, meta);
                count += 1;
            }
        }
    });
    count
}

/// Update canister metadata (controller only)
/// Only provided fields are updated; None means unchanged
#[ic_cdk::update]
fn update_canister(canister_id: Principal, updates: CanisterUpdate) -> bool {
    if !is_controller() {
        ic_cdk::trap("Only controller can update canisters");
    }

    let key = PrincipalKey::new(canister_id);
    CANISTERS.with(|c| {
        let mut map = c.borrow_mut();
        if let Some(mut meta) = map.get(&key) {
            if let Some(proxy_id) = updates.proxy_id {
                meta.proxy_id = proxy_id;
            }
            if let Some(proxy_type) = updates.proxy_type {
                meta.proxy_type = proxy_type;
            }
            if let Some(project) = updates.project {
                meta.project_name = sanitize_project_name(project);
            }
            if let Some(website) = updates.website {
                meta.website = sanitize_website(website);
            }
            map.insert(key, meta);
            true
        } else {
            false
        }
    })
}

/// Remove a single canister and its snapshots (controller only)
#[ic_cdk::update]
fn remove_canister(canister_id: Principal) -> bool {
    if !is_controller() {
        ic_cdk::trap("Only controller can remove canisters");
    }

    let key = PrincipalKey::new(canister_id);

    // First check if canister exists
    let exists = CANISTERS.with(|c| c.borrow().contains_key(&key));
    if !exists {
        return false;
    }

    // Delete snapshots first
    delete_snapshots_for_canister(&key);

    // Then remove the canister
    CANISTERS.with(|c| {
        c.borrow_mut().remove(&key);
    });

    true
}

/// Remove multiple canisters and their snapshots (controller only)
#[ic_cdk::update]
fn remove_canisters(canister_ids: Vec<Principal>) -> u64 {
    if !is_controller() {
        ic_cdk::trap("Only controller can remove canisters");
    }

    let mut count = 0u64;
    for canister_id in canister_ids {
        let key = PrincipalKey::new(canister_id);

        let exists = CANISTERS.with(|c| c.borrow().contains_key(&key));
        if exists {
            delete_snapshots_for_canister(&key);
            CANISTERS.with(|c| {
                c.borrow_mut().remove(&key);
            });
            count += 1;
        }
    }
    count
}

/// Take a snapshot of all canisters (controller only)
#[ic_cdk::update]
async fn take_snapshot() -> SnapshotResult {
    if !is_controller() {
        ic_cdk::trap("Only controller can trigger snapshots");
    }
    do_take_snapshot().await
}

/// Internal snapshot logic - called by timer and public API
async fn do_take_snapshot() -> SnapshotResult {
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

    // Prune old snapshots (keep RETENTION_PERIOD = 30 days)
    // Note: We must check ALL entries because SnapshotKey sorts by (canister_id, timestamp),
    // not globally by timestamp. Early break would miss old snapshots for later-sorted canisters.
    let cutoff = timestamp.saturating_sub(RETENTION_PERIOD);
    let pruned = SNAPSHOTS.with(|s| {
        let mut map = s.borrow_mut();
        let to_remove: Vec<_> = map
            .iter()
            .filter(|(key, _)| key.timestamp < cutoff)
            .map(|(key, _)| key.clone())
            .collect();

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

    // Clear snapshots first to avoid orphaned data
    do_clear_snapshots();

    CANISTERS.with(|c| {
        let mut map = c.borrow_mut();
        let keys: Vec<_> = map.iter().map(|(k, _)| k.clone()).collect();
        for key in keys {
            map.remove(&key);
        }
    });
}

/// Internal helper to clear all snapshots
fn do_clear_snapshots() {
    SNAPSHOTS.with(|s| {
        let mut map = s.borrow_mut();
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
    do_clear_snapshots();
}

// =============================================================================
// Timer Functions
// =============================================================================

fn schedule_snapshot_timer() {
    let timer_id = ic_cdk_timers::set_timer_interval(SNAPSHOT_INTERVAL, || {
        ic_cdk::spawn(async {
            let result = do_take_snapshot().await;
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
