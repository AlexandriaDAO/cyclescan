# CycleScan Backup & Restore

## Files

- `backup_metadata.sh` - Export all canister metadata to JSON
- `batch_import.py` - Restore metadata from JSON backup (handles large datasets)
- `restore_metadata.sh` - Simple restore (fails for large datasets)
- `canister_metadata_backup.json` - Latest backup

## Backup

```bash
cd data/backup
./backup_metadata.sh
```

This exports project names, websites, proxy IDs, and proxy types for all canisters.

## Restore (Emergency)

### Option 1: dfx Canister Snapshot (Full state)

Restores complete canister state including cycle history:

```bash
# List available snapshots
dfx canister snapshot list cyclescan_backend --network ic

# Restore from snapshot (stops canister automatically)
dfx canister snapshot load cyclescan_backend <snapshot_id> --network ic
dfx canister start cyclescan_backend --network ic
```

### Option 2: JSON Backup (Metadata only)

Restores project metadata but not cycle history:

```bash
# Clear existing data (optional)
dfx canister call cyclescan_backend clear_canisters --network ic
dfx canister call cyclescan_backend clear_snapshots --network ic

# Restore from backup
cd data/backup
python3 batch_import.py

# Take first cycle snapshot
dfx canister call cyclescan_backend take_snapshot --network ic
```

## Creating a dfx Snapshot

```bash
dfx canister stop cyclescan_backend --network ic
dfx canister snapshot create cyclescan_backend --network ic
dfx canister start cyclescan_backend --network ic
```

Current snapshot: `0000000000000000000000000041006f0101`
