# Canister Research Prompt

Research and add subcategory labels for the **[PROJECT_NAME]** project's canisters.

## Task
Identify what each canister does and add subcategory metadata to the JSON files.

## Steps
1. Get project canisters:
   ```bash
   grep -B2 -A10 '"[PROJECT_NAME]"' data/archive/canisters_backup.json
   ```

2. For each canister, get its candid interface:
   ```bash
   dfx canister metadata <canister-id> candid:service --network ic
   ```

   If 403 error, probe with common methods (see CLAUDE.md for full list).

3. Update `data/archive/projects_backup.json` - add `subcategory_descriptions` to project entry

4. Update `data/archive/canisters_backup.json` - add `"subcategory"` field to each identified canister

## Rules
- Query canisters directly with dfx - do NOT use web searches
- Only label canisters you identify with certainty
- Skip unknown canisters (no subcategory field)
- Descriptions: one static line, no changing values

See "Researching Canister Functions" section in CLAUDE.md for full methodology and examples.
