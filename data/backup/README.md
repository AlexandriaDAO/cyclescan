# CycleScan Backup & Restore

## Files

- `backup_metadata.sh` - Export projects + canisters to JSON
- `batch_import.py` - Restore from JSON backup
- `projects_backup.json` - Project metadata (name, website)
- `canisters_backup.json` - Canister metadata (id, proxy, project ref)

## Backup

```bash
./backup_metadata.sh
```

Exports two files:
- `projects_backup.json` - Projects with websites
- `canisters_backup.json` - Canisters (no website, references project)

## Restore

After a reinstall or fresh deployment:

```bash
python3 batch_import.py
dfx canister call cyclescan_backend take_snapshot --network ic
```

Note: Pre-computed values (balance, burns) are not backed up. They populate on first snapshot.

## dfx Snapshots

For full state backup (including cycle history):

```bash
# Create snapshot
dfx canister stop cyclescan_backend --network ic
dfx canister snapshot create cyclescan_backend --network ic
dfx canister start cyclescan_backend --network ic

# Restore from snapshot
dfx canister snapshot load cyclescan_backend <snapshot_id> --network ic
```

## Migration from Old Format

If you have the old `canister_metadata_backup.json` format:

```bash
python3 convert_legacy.py  # One-time conversion
python3 batch_import.py
dfx canister call cyclescan_backend take_snapshot --network ic
```

Then delete `canister_metadata_backup.json` and `convert_legacy.py`.
