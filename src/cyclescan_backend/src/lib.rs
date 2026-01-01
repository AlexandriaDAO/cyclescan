use candid::{CandidType, Deserialize, Principal};
use ic_cdk::{api::time, caller, init, post_upgrade, query, update};
use ic_cdk_timers::TimerId;
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    storable::Bound,
    DefaultMemoryImpl, StableBTreeMap, Storable,
};
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;

// ============================================================================
// CONSTANTS
// ============================================================================

const NANOS_PER_HOUR: u64 = 3_600_000_000_000;
const NANOS_PER_DAY: u64 = 86_400_000_000_000;
const SEVEN_DAYS_NANOS: u64 = 7 * NANOS_PER_DAY;
const THIRTY_DAYS_NANOS: u64 = 30 * NANOS_PER_DAY;
const QUERY_BATCH_SIZE: usize = 50;
const MAX_PROJECT_BYTES: usize = 100;
const MAX_WEBSITE_BYTES: usize = 200;

// ============================================================================
// PROXY TYPES
// ============================================================================

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum ProxyType {
    Blackhole,
    SnsRoot,
}

impl ProxyType {
    fn to_byte(&self) -> u8 {
        match self {
            ProxyType::Blackhole => 0,
            ProxyType::SnsRoot => 1,
        }
    }

    fn from_byte(b: u8) -> Self {
        match b {
            1 => ProxyType::SnsRoot,
            _ => ProxyType::Blackhole,
        }
    }
}

// ============================================================================
// API TYPES
// ============================================================================

#[derive(CandidType, Deserialize)]
pub struct CanisterImport {
    pub canister_id: Principal,
    pub proxy_id: Principal,
    pub proxy_type: ProxyType,
    pub project: Option<String>,
    pub website: Option<String>,
    pub valid: Option<bool>,
}

#[derive(CandidType, Deserialize)]
pub struct CanisterUpdate {
    pub project: Option<Option<String>>,
    pub website: Option<Option<String>>,
}

#[derive(CandidType, Clone)]
pub struct LeaderboardEntry {
    pub canister_id: Principal,
    pub project: Option<String>,
    pub website: Option<String>,
    pub balance: u128,
    pub burn_1h: Option<u128>,
    pub burn_24h: Option<u128>,
    pub burn_7d: Option<u128>,
}

#[derive(CandidType)]
pub struct LeaderboardPage {
    pub entries: Vec<LeaderboardEntry>,
    pub total: u64,
    pub offset: u64,
}

#[derive(CandidType)]
pub struct ProjectEntry {
    pub project: String,
    pub website: Option<String>,
    pub canister_count: u64,
    pub total_balance: u128,
    pub total_burn_1h: Option<u128>,
    pub total_burn_24h: Option<u128>,
    pub total_burn_7d: Option<u128>,
}

#[derive(CandidType)]
pub struct CanisterDetail {
    pub canister_id: Principal,
    pub proxy_id: Principal,
    pub proxy_type: ProxyType,
    pub project: Option<String>,
    pub website: Option<String>,
    pub current_balance: u128,
    pub burn_1h: Option<u128>,
    pub burn_24h: Option<u128>,
    pub burn_7d: Option<u128>,
    pub burn_30d: Option<u128>,
    pub snapshots: Vec<Snapshot>,
}

#[derive(CandidType, Clone)]
pub struct Snapshot {
    pub timestamp: u64,
    pub cycles: u128,
}

#[derive(CandidType)]
pub struct Stats {
    pub canister_count: u64,
    pub snapshot_count: u64,
    pub tracked_projects: u64,
}

#[derive(CandidType)]
pub struct SnapshotResult {
    pub success: u64,
    pub failed: u64,
    pub pruned: u64,
}

// ============================================================================
// STORAGE TYPES
// ============================================================================

#[derive(Clone)]
struct PrincipalKey([u8; 30]);

impl PrincipalKey {
    fn from_principal(p: Principal) -> Self {
        let mut bytes = [0u8; 30];
        let slice = p.as_slice();
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
    fn to_bytes(&self) -> Cow<[u8]> {
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

impl PartialEq for PrincipalKey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for PrincipalKey {}
impl PartialOrd for PrincipalKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for PrincipalKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

struct CanisterMeta {
    proxy_id: Principal,
    proxy_type: ProxyType,
    project: Option<String>,
    website: Option<String>,
    valid: bool,
}

// Fixed layout: proxy_len(1) + proxy(29) + type(1) + proj_len(2) + proj(100) + web_len(2) + web(200) + valid(1) = 336
const CANISTER_META_SIZE: u32 = 336;

impl Storable for CanisterMeta {
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut buf = vec![0u8; CANISTER_META_SIZE as usize];
        let proxy_slice = self.proxy_id.as_slice();
        buf[0] = proxy_slice.len() as u8;
        buf[1..1 + proxy_slice.len()].copy_from_slice(proxy_slice);
        buf[30] = self.proxy_type.to_byte();

        if let Some(ref p) = self.project {
            let bytes = p.as_bytes();
            let len = bytes.len().min(MAX_PROJECT_BYTES);
            buf[31] = (len >> 8) as u8;
            buf[32] = len as u8;
            buf[33..33 + len].copy_from_slice(&bytes[..len]);
        }

        if let Some(ref w) = self.website {
            let bytes = w.as_bytes();
            let len = bytes.len().min(MAX_WEBSITE_BYTES);
            buf[133] = (len >> 8) as u8;
            buf[134] = len as u8;
            buf[135..135 + len].copy_from_slice(&bytes[..len]);
        }

        buf[335] = if self.valid { 1 } else { 0 };

        Cow::Owned(buf)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let proxy_len = (bytes[0] as usize).min(29);
        let proxy_id = Principal::from_slice(&bytes[1..1 + proxy_len]);
        let proxy_type = ProxyType::from_byte(bytes[30]);

        let proj_len = (((bytes[31] as usize) << 8) | (bytes[32] as usize)).min(MAX_PROJECT_BYTES);
        let project = if proj_len > 0 {
            String::from_utf8(bytes[33..33 + proj_len].to_vec()).ok()
        } else {
            None
        };

        let web_len = (((bytes[133] as usize) << 8) | (bytes[134] as usize)).min(MAX_WEBSITE_BYTES);
        let website = if web_len > 0 {
            String::from_utf8(bytes[135..135 + web_len].to_vec()).ok()
        } else {
            None
        };

        // Default to true for backward compatibility with old 335-byte records
        let valid = if bytes.len() > 335 { bytes[335] != 0 } else { true };

        Self { proxy_id, proxy_type, project, website, valid }
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: CANISTER_META_SIZE,
        is_fixed_size: true,
    };
}

#[derive(Clone)]
struct SnapshotKey {
    canister: PrincipalKey,
    timestamp: u64,
}

impl Storable for SnapshotKey {
    fn to_bytes(&self) -> Cow<[u8]> {
        let mut buf = [0u8; 38];
        buf[..30].copy_from_slice(&self.canister.0);
        buf[30..38].copy_from_slice(&self.timestamp.to_be_bytes());
        Cow::Owned(buf.to_vec())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let mut canister_bytes = [0u8; 30];
        canister_bytes.copy_from_slice(&bytes[..30]);
        let mut ts_bytes = [0u8; 8];
        ts_bytes.copy_from_slice(&bytes[30..38]);
        Self {
            canister: PrincipalKey(canister_bytes),
            timestamp: u64::from_be_bytes(ts_bytes),
        }
    }
    const BOUND: Bound = Bound::Bounded {
        max_size: 38,
        is_fixed_size: true,
    };
}

impl PartialEq for SnapshotKey {
    fn eq(&self, other: &Self) -> bool {
        self.canister == other.canister && self.timestamp == other.timestamp
    }
}
impl Eq for SnapshotKey {}
impl PartialOrd for SnapshotKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for SnapshotKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.canister.cmp(&other.canister) {
            std::cmp::Ordering::Equal => self.timestamp.cmp(&other.timestamp),
            other => other,
        }
    }
}

struct CyclesValue(u128);

impl Storable for CyclesValue {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(self.0.to_be_bytes().to_vec())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let mut arr = [0u8; 16];
        arr.copy_from_slice(&bytes);
        Self(u128::from_be_bytes(arr))
    }
    const BOUND: Bound = Bound::Bounded {
        max_size: 16,
        is_fixed_size: true,
    };
}

// ============================================================================
// MEMORY MANAGEMENT
// ============================================================================

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

    static TIMER_ID: RefCell<Option<TimerId>> = RefCell::new(None);
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn is_controller() -> bool {
    ic_cdk::api::is_controller(&caller())
}

fn truncate_utf8(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_string()
}

fn get_latest_balance(key: &PrincipalKey) -> u128 {
    SNAPSHOTS.with(|s| {
        let map = s.borrow();
        let start = SnapshotKey { canister: key.clone(), timestamp: 0 };
        let end = SnapshotKey { canister: key.clone(), timestamp: u64::MAX };
        map.range(start..=end).last().map(|(_, v)| v.0).unwrap_or(0)
    })
}

fn get_snapshots(key: &PrincipalKey) -> Vec<(u64, u128)> {
    SNAPSHOTS.with(|s| {
        let map = s.borrow();
        let start = SnapshotKey { canister: key.clone(), timestamp: 0 };
        let end = SnapshotKey { canister: key.clone(), timestamp: u64::MAX };
        map.range(start..=end).map(|(k, v)| (k.timestamp, v.0)).collect()
    })
}

fn calculate_burn(key: &PrincipalKey, window_nanos: u64) -> Option<u128> {
    let now = time();
    let cutoff = now.saturating_sub(window_nanos);
    let snapshots = get_snapshots(key);

    if snapshots.len() < 2 {
        return None;
    }

    // Find snapshots within window
    let in_window: Vec<_> = snapshots.iter().filter(|(ts, _)| *ts >= cutoff).collect();

    if in_window.len() >= 2 {
        // Calculate actual burn from snapshots (ignoring top-ups)
        let mut total_burn: u128 = 0;
        for i in 1..in_window.len() {
            let prev = in_window[i - 1].1;
            let curr = in_window[i].1;
            if curr < prev {
                total_burn += prev - curr;
            }
        }
        Some(total_burn)
    } else {
        // Extrapolate from last 2 snapshots
        let len = snapshots.len();
        let (ts1, c1) = snapshots[len - 2];
        let (ts2, c2) = snapshots[len - 1];

        if ts2 <= ts1 || c2 >= c1 {
            return Some(0);
        }

        let elapsed = ts2 - ts1;
        let burn = c1 - c2;
        let rate = burn as f64 / elapsed as f64;
        Some((rate * window_nanos as f64) as u128)
    }
}

fn build_entry(key: &PrincipalKey, meta: &CanisterMeta) -> LeaderboardEntry {
    LeaderboardEntry {
        canister_id: key.to_principal(),
        project: meta.project.clone(),
        website: meta.website.clone(),
        balance: get_latest_balance(key),
        burn_1h: calculate_burn(key, NANOS_PER_HOUR),
        burn_24h: calculate_burn(key, NANOS_PER_DAY),
        burn_7d: calculate_burn(key, SEVEN_DAYS_NANOS),
    }
}

// ============================================================================
// QUERY ENDPOINTS
// ============================================================================

#[query]
fn get_leaderboard(offset: u64, limit: u64) -> LeaderboardPage {
    let limit = limit.min(1000) as usize;
    let offset = offset as usize;

    CANISTERS.with(|c| {
        let canisters = c.borrow();
        let mut entries: Vec<LeaderboardEntry> = canisters
            .iter()
            .filter(|(_, meta)| meta.valid)
            .map(|(key, meta)| build_entry(&key, &meta))
            .collect();

        // Sort by 24h burn descending
        entries.sort_by(|a, b| {
            b.burn_24h.unwrap_or(0).cmp(&a.burn_24h.unwrap_or(0))
        });

        let total = entries.len() as u64;
        let page = entries.into_iter().skip(offset).take(limit).collect();

        LeaderboardPage {
            entries: page,
            total,
            offset: offset as u64,
        }
    })
}

#[query]
fn get_project_leaderboard() -> Vec<ProjectEntry> {
    CANISTERS.with(|c| {
        let canisters = c.borrow();
        // (count, balance, burn_1h, burn_24h, burn_7d, website)
        let mut projects: HashMap<String, (u64, u128, u128, u128, u128, Option<String>)> = HashMap::new();

        for (key, meta) in canisters.iter() {
            if !meta.valid {
                continue;
            }
            if let Some(ref project) = meta.project {
                let balance = get_latest_balance(&key);
                let burn_1h = calculate_burn(&key, NANOS_PER_HOUR).unwrap_or(0);
                let burn_24h = calculate_burn(&key, NANOS_PER_DAY).unwrap_or(0);
                let burn_7d = calculate_burn(&key, SEVEN_DAYS_NANOS).unwrap_or(0);
                let entry = projects.entry(project.clone()).or_insert((0, 0, 0, 0, 0, None));
                entry.0 += 1;
                entry.1 += balance;
                entry.2 += burn_1h;
                entry.3 += burn_24h;
                entry.4 += burn_7d;
                // Keep first website found for this project
                if entry.5.is_none() && meta.website.is_some() {
                    entry.5 = meta.website.clone();
                }
            }
        }

        let mut result: Vec<ProjectEntry> = projects
            .into_iter()
            .map(|(project, (count, balance, burn_1h, burn_24h, burn_7d, website))| ProjectEntry {
                project,
                website,
                canister_count: count,
                total_balance: balance,
                total_burn_1h: if burn_1h > 0 { Some(burn_1h) } else { None },
                total_burn_24h: if burn_24h > 0 { Some(burn_24h) } else { None },
                total_burn_7d: if burn_7d > 0 { Some(burn_7d) } else { None },
            })
            .collect();

        result.sort_by(|a, b| b.total_burn_24h.cmp(&a.total_burn_24h));
        result
    })
}

#[query]
fn get_project_canisters(project_name: String) -> Vec<LeaderboardEntry> {
    CANISTERS.with(|c| {
        let canisters = c.borrow();
        let mut entries: Vec<LeaderboardEntry> = canisters
            .iter()
            .filter(|(_, meta)| meta.valid && meta.project.as_ref() == Some(&project_name))
            .map(|(key, meta)| build_entry(&key, &meta))
            .collect();

        // Sort by 24h burn descending
        entries.sort_by(|a, b| {
            b.burn_24h.unwrap_or(0).cmp(&a.burn_24h.unwrap_or(0))
        });

        entries
    })
}

#[query]
fn get_canister(canister_id: Principal) -> Option<CanisterDetail> {
    let key = PrincipalKey::from_principal(canister_id);

    CANISTERS.with(|c| {
        c.borrow().get(&key).map(|meta| {
            let snapshots: Vec<Snapshot> = get_snapshots(&key)
                .into_iter()
                .map(|(ts, cycles)| Snapshot { timestamp: ts, cycles })
                .collect();

            CanisterDetail {
                canister_id,
                proxy_id: meta.proxy_id,
                proxy_type: meta.proxy_type,
                project: meta.project,
                website: meta.website,
                current_balance: get_latest_balance(&key),
                burn_1h: calculate_burn(&key, NANOS_PER_HOUR),
                burn_24h: calculate_burn(&key, NANOS_PER_DAY),
                burn_7d: calculate_burn(&key, SEVEN_DAYS_NANOS),
                burn_30d: calculate_burn(&key, THIRTY_DAYS_NANOS),
                snapshots,
            }
        })
    })
}

#[query]
fn get_stats() -> Stats {
    let canister_count = CANISTERS.with(|c| c.borrow().len());
    let snapshot_count = SNAPSHOTS.with(|s| s.borrow().len());
    let tracked_projects = CANISTERS.with(|c| {
        c.borrow()
            .iter()
            .filter_map(|(_, meta)| meta.project)
            .collect::<std::collections::HashSet<_>>()
            .len() as u64
    });

    Stats { canister_count, snapshot_count, tracked_projects }
}

#[query]
fn is_timer_running() -> bool {
    TIMER_ID.with(|t| t.borrow().is_some())
}

#[query]
fn export_canisters() -> Vec<CanisterImport> {
    CANISTERS.with(|c| {
        c.borrow()
            .iter()
            .map(|(key, meta)| CanisterImport {
                canister_id: key.to_principal(),
                proxy_id: meta.proxy_id,
                proxy_type: meta.proxy_type,
                project: meta.project,
                website: meta.website,
                valid: Some(meta.valid),
            })
            .collect()
    })
}

// ============================================================================
// UPDATE ENDPOINTS
// ============================================================================

#[update]
fn import_canisters(canisters: Vec<CanisterImport>) -> u64 {
    assert!(is_controller(), "Not authorized");

    let mut count = 0u64;
    CANISTERS.with(|c| {
        let mut map = c.borrow_mut();
        for import in canisters {
            let key = PrincipalKey::from_principal(import.canister_id);
            let meta = CanisterMeta {
                proxy_id: import.proxy_id,
                proxy_type: import.proxy_type,
                project: import.project.map(|p| truncate_utf8(&p, MAX_PROJECT_BYTES)),
                website: import.website.map(|w| truncate_utf8(&w, MAX_WEBSITE_BYTES)),
                valid: import.valid.unwrap_or(true),
            };
            map.insert(key, meta);
            count += 1;
        }
    });
    count
}

#[update]
fn update_canister(canister_id: Principal, updates: CanisterUpdate) {
    assert!(is_controller(), "Not authorized");

    let key = PrincipalKey::from_principal(canister_id);
    CANISTERS.with(|c| {
        let mut map = c.borrow_mut();
        if let Some(mut meta) = map.get(&key) {
            if let Some(project) = updates.project {
                meta.project = project.map(|p| truncate_utf8(&p, MAX_PROJECT_BYTES));
            }
            if let Some(website) = updates.website {
                meta.website = website.map(|w| truncate_utf8(&w, MAX_WEBSITE_BYTES));
            }
            map.insert(key, meta);
        }
    });
}

#[update]
fn set_valid(canister_id: Principal, valid: bool) -> bool {
    assert!(is_controller(), "Not authorized");

    let key = PrincipalKey::from_principal(canister_id);
    CANISTERS.with(|c| {
        let mut map = c.borrow_mut();
        if let Some(mut meta) = map.get(&key) {
            meta.valid = valid;
            map.insert(key, meta);
            true
        } else {
            false
        }
    })
}

#[update]
fn remove_canisters(canister_ids: Vec<Principal>) {
    assert!(is_controller(), "Not authorized");

    for canister_id in canister_ids {
        let key = PrincipalKey::from_principal(canister_id);

        // Remove metadata
        CANISTERS.with(|c| c.borrow_mut().remove(&key));

        // Remove snapshots
        let snapshots = get_snapshots(&key);
        SNAPSHOTS.with(|s| {
            let mut map = s.borrow_mut();
            for (ts, _) in snapshots {
                map.remove(&SnapshotKey { canister: key.clone(), timestamp: ts });
            }
        });
    }
}

#[update]
fn clear_all() {
    assert!(is_controller(), "Not authorized");

    CANISTERS.with(|c| {
        let mut map = c.borrow_mut();
        let keys: Vec<_> = map.iter().map(|(k, _)| k).collect();
        for key in keys {
            map.remove(&key);
        }
    });

    SNAPSHOTS.with(|s| {
        let mut map = s.borrow_mut();
        let keys: Vec<_> = map.iter().map(|(k, _)| k).collect();
        for key in keys {
            map.remove(&key);
        }
    });
}

#[update]
async fn take_snapshot() -> SnapshotResult {
    assert!(is_controller(), "Not authorized");
    do_take_snapshot().await
}

#[update]
fn start_timer() {
    assert!(is_controller(), "Not authorized");
    setup_timer();
}

#[update]
fn stop_timer() {
    assert!(is_controller(), "Not authorized");
    TIMER_ID.with(|t| {
        if let Some(id) = t.borrow_mut().take() {
            ic_cdk_timers::clear_timer(id);
        }
    });
}

// ============================================================================
// SNAPSHOT LOGIC
// ============================================================================

async fn do_take_snapshot() -> SnapshotResult {
    let now = time();
    let mut success = 0u64;
    let mut failed = 0u64;

    // Group canisters by proxy type
    let (blackhole_canisters, sns_groups) = CANISTERS.with(|c| {
        let canisters = c.borrow();
        let mut blackhole: Vec<(PrincipalKey, Principal)> = Vec::new();
        let mut sns: HashMap<Principal, Vec<PrincipalKey>> = HashMap::new();

        for (key, meta) in canisters.iter() {
            match meta.proxy_type {
                ProxyType::Blackhole => {
                    blackhole.push((key, meta.proxy_id));
                }
                ProxyType::SnsRoot => {
                    sns.entry(meta.proxy_id).or_default().push(key);
                }
            }
        }
        (blackhole, sns)
    });

    // Query blackhole canisters in batches
    for batch in blackhole_canisters.chunks(QUERY_BATCH_SIZE) {
        let futures: Vec<_> = batch
            .iter()
            .map(|(key, proxy_id)| {
                let canister_id = key.to_principal();
                let proxy = *proxy_id;
                async move {
                    let result = query_blackhole(proxy, canister_id).await;
                    (key.clone(), result)
                }
            })
            .collect();

        let results = futures::future::join_all(futures).await;

        SNAPSHOTS.with(|s| {
            let mut map = s.borrow_mut();
            for (key, result) in results {
                match result {
                    Ok(cycles) => {
                        map.insert(SnapshotKey { canister: key, timestamp: now }, CyclesValue(cycles));
                        success += 1;
                    }
                    Err(_) => failed += 1,
                }
            }
        });
    }

    // Query SNS roots
    for (sns_root, canister_keys) in sns_groups {
        match query_sns_root(sns_root).await {
            Ok(cycles_map) => {
                SNAPSHOTS.with(|s| {
                    let mut map = s.borrow_mut();
                    for key in canister_keys {
                        let canister_id = key.to_principal();
                        if let Some(&cycles) = cycles_map.get(&canister_id) {
                            map.insert(SnapshotKey { canister: key, timestamp: now }, CyclesValue(cycles));
                            success += 1;
                        } else {
                            failed += 1;
                        }
                    }
                });
            }
            Err(_) => {
                failed += canister_keys.len() as u64;
            }
        }
    }

    // Prune old snapshots
    let cutoff = now.saturating_sub(THIRTY_DAYS_NANOS);
    let pruned = SNAPSHOTS.with(|s| {
        let mut map = s.borrow_mut();
        let to_remove: Vec<_> = map
            .iter()
            .filter(|(k, _)| k.timestamp < cutoff)
            .map(|(k, _)| k)
            .collect();
        let count = to_remove.len() as u64;
        for key in to_remove {
            map.remove(&key);
        }
        count
    });

    SnapshotResult { success, failed, pruned }
}

// Blackhole canister_status query
#[derive(CandidType, Deserialize)]
struct CanisterIdRecord {
    canister_id: Principal,
}

#[derive(CandidType, Deserialize)]
struct CanisterStatusResponse {
    cycles: candid::Nat,
}

async fn query_blackhole(proxy: Principal, canister_id: Principal) -> Result<u128, String> {
    let args = CanisterIdRecord { canister_id };
    let result: Result<(CanisterStatusResponse,), _> =
        ic_cdk::call(proxy, "canister_status", (args,)).await;

    match result {
        Ok((status,)) => {
            let cycles_str = status.cycles.0.to_string();
            cycles_str.parse::<u128>().map_err(|e| e.to_string())
        }
        Err((code, msg)) => Err(format!("{:?}: {}", code, msg)),
    }
}

// SNS root get_sns_canisters_summary query
#[derive(CandidType, Deserialize)]
struct GetSnsCanistersSummaryRequest {}

#[derive(CandidType, Deserialize)]
struct GetSnsCanistersSummaryResponse {
    root: Option<CanisterSummary>,
    governance: Option<CanisterSummary>,
    ledger: Option<CanisterSummary>,
    swap: Option<CanisterSummary>,
    index: Option<CanisterSummary>,
    archives: Vec<CanisterSummary>,
    dapps: Vec<CanisterSummary>,
}

#[derive(CandidType, Deserialize)]
struct CanisterSummary {
    canister_id: Option<Principal>,
    status: Option<CanisterStatusResult>,
}

#[derive(CandidType, Deserialize)]
struct CanisterStatusResult {
    cycles: candid::Nat,
}

async fn query_sns_root(sns_root: Principal) -> Result<HashMap<Principal, u128>, String> {
    let args = GetSnsCanistersSummaryRequest {};
    let result: Result<(GetSnsCanistersSummaryResponse,), _> =
        ic_cdk::call(sns_root, "get_sns_canisters_summary", (args,)).await;

    match result {
        Ok((response,)) => {
            let mut cycles_map = HashMap::new();

            let all_canisters = [
                response.root,
                response.governance,
                response.ledger,
                response.swap,
                response.index,
            ]
            .into_iter()
            .flatten()
            .chain(response.archives)
            .chain(response.dapps);

            for summary in all_canisters {
                if let (Some(id), Some(status)) = (summary.canister_id, summary.status) {
                    let cycles_str = status.cycles.0.to_string();
                    if let Ok(cycles) = cycles_str.parse::<u128>() {
                        cycles_map.insert(id, cycles);
                    }
                }
            }

            Ok(cycles_map)
        }
        Err((code, msg)) => Err(format!("{:?}: {}", code, msg)),
    }
}

// ============================================================================
// TIMER MANAGEMENT
// ============================================================================

fn setup_timer() {
    TIMER_ID.with(|t| {
        if t.borrow().is_some() {
            return;
        }
        let id = ic_cdk_timers::set_timer_interval(
            std::time::Duration::from_nanos(NANOS_PER_HOUR),
            || ic_cdk::spawn(async { do_take_snapshot().await; }),
        );
        *t.borrow_mut() = Some(id);
    });
}

// ============================================================================
// LIFECYCLE
// ============================================================================

#[init]
fn init() {
    setup_timer();
}

#[post_upgrade]
fn post_upgrade() {
    setup_timer();
}

// ============================================================================
// CANDID EXPORT
// ============================================================================

ic_cdk::export_candid!();
