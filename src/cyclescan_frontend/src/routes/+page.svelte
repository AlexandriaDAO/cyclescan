<script>
  import "../index.scss";
  import { onMount } from "svelte";
  import { backend } from "$lib/canisters";

  let entries = [];
  let stats = null;
  let loading = true;
  let error = null;
  let searchQuery = "";
  let sortColumn = "burn_24h";
  let sortDirection = "desc";

  const TRILLION = 1_000_000_000_000n;
  const BILLION = 1_000_000_000n;
  const MILLION = 1_000_000n;

  function formatCycles(value) {
    if (value === null || value === undefined) return null;
    const n = BigInt(value);
    if (n >= TRILLION) {
      return (Number(n / BILLION) / 1000).toFixed(2) + "T";
    } else if (n >= BILLION) {
      return (Number(n / MILLION) / 1000).toFixed(2) + "B";
    } else if (n >= MILLION) {
      return (Number(n) / 1_000_000).toFixed(2) + "M";
    } else {
      return Number(n).toLocaleString();
    }
  }

  function formatBurn(value) {
    if (value === null || value === undefined || value.length === 0) {
      return { text: "-", class: "no-data" };
    }
    const v = value[0];
    if (v === 0n || v === 0) {
      return { text: "0", class: "zero" };
    }
    return { text: formatCycles(v), class: "positive" };
  }

  function shortenCanisterId(id) {
    const s = id.toString();
    if (s.length <= 15) return s;
    return s.slice(0, 5) + "..." + s.slice(-3);
  }

  function dashboardUrl(id) {
    return `https://dashboard.internetcomputer.org/canister/${id}`;
  }

  function sortBy(column) {
    if (sortColumn === column) {
      sortDirection = sortDirection === "desc" ? "asc" : "desc";
    } else {
      sortColumn = column;
      sortDirection = "desc";
    }
  }

  function getValue(entry, col) {
    switch (col) {
      case "balance": return entry.balance;
      case "burn_1h": return entry.burn_1h?.[0] ?? -1n;
      case "burn_24h": return entry.burn_24h?.[0] ?? -1n;
      case "burn_7d": return entry.burn_7d?.[0] ?? -1n;
      case "project": return entry.project?.[0] ?? "";
      case "canister_id": return entry.canister_id.toString();
      default: return 0;
    }
  }

  $: filteredEntries = entries.filter(e => {
    if (!searchQuery) return true;
    const q = searchQuery.toLowerCase();
    const id = e.canister_id.toString().toLowerCase();
    const project = (e.project?.[0] ?? "").toLowerCase();
    return id.includes(q) || project.includes(q);
  });

  $: sortedEntries = [...filteredEntries].sort((a, b) => {
    const aVal = getValue(a, sortColumn);
    const bVal = getValue(b, sortColumn);
    let cmp = 0;
    if (typeof aVal === "string") {
      cmp = aVal.localeCompare(bVal);
    } else {
      if (aVal < bVal) cmp = -1;
      else if (aVal > bVal) cmp = 1;
    }
    return sortDirection === "desc" ? -cmp : cmp;
  });

  function formatNumber(n) {
    return Number(n).toLocaleString();
  }

  onMount(async () => {
    try {
      const [leaderboard, statsData] = await Promise.all([
        backend.get_leaderboard(),
        backend.get_stats()
      ]);
      entries = leaderboard;
      stats = statsData;
      loading = false;
    } catch (e) {
      error = e.message || "Failed to load data";
      loading = false;
    }
  });
</script>

<div class="container">
  <header>
    <h1>CycleScan</h1>
    {#if stats}
      <div class="stats">
        <span>Canisters: <strong>{formatNumber(stats.canister_count)}</strong></span>
        <span>Snapshots: <strong>{formatNumber(stats.snapshot_count)}</strong></span>
      </div>
    {/if}
  </header>

  <div class="controls">
    <input
      type="text"
      class="search"
      placeholder="Search by canister ID or project..."
      bind:value={searchQuery}
    />
  </div>

  {#if loading}
    <div class="loading">Loading leaderboard...</div>
  {:else if error}
    <div class="error">Error: {error}</div>
  {:else if sortedEntries.length === 0}
    <div class="empty-state">
      {#if searchQuery}
        No canisters match your search.
      {:else}
        No canisters tracked yet.
      {/if}
    </div>
  {:else}
    <div class="table-wrapper">
      <table>
        <thead>
          <tr>
            <th class="rank">#</th>
            <th
              class:sorted={sortColumn === "canister_id"}
              on:click={() => sortBy("canister_id")}
            >
              Canister
              <span class="sort-arrow">{sortColumn === "canister_id" ? (sortDirection === "desc" ? "▼" : "▲") : "▼"}</span>
            </th>
            <th
              class:sorted={sortColumn === "project"}
              on:click={() => sortBy("project")}
            >
              Project
              <span class="sort-arrow">{sortColumn === "project" ? (sortDirection === "desc" ? "▼" : "▲") : "▼"}</span>
            </th>
            <th
              class:sorted={sortColumn === "balance"}
              on:click={() => sortBy("balance")}
            >
              Balance
              <span class="sort-arrow">{sortColumn === "balance" ? (sortDirection === "desc" ? "▼" : "▲") : "▼"}</span>
            </th>
            <th
              class:sorted={sortColumn === "burn_1h"}
              on:click={() => sortBy("burn_1h")}
            >
              1h Burn
              <span class="sort-arrow">{sortColumn === "burn_1h" ? (sortDirection === "desc" ? "▼" : "▲") : "▼"}</span>
            </th>
            <th
              class:sorted={sortColumn === "burn_24h"}
              on:click={() => sortBy("burn_24h")}
            >
              24h Burn
              <span class="sort-arrow">{sortColumn === "burn_24h" ? (sortDirection === "desc" ? "▼" : "▲") : "▼"}</span>
            </th>
            <th
              class:sorted={sortColumn === "burn_7d"}
              on:click={() => sortBy("burn_7d")}
            >
              7d Burn
              <span class="sort-arrow">{sortColumn === "burn_7d" ? (sortDirection === "desc" ? "▼" : "▲") : "▼"}</span>
            </th>
          </tr>
        </thead>
        <tbody>
          {#each sortedEntries as entry, i}
            <tr>
              <td class="rank">{i + 1}</td>
              <td class="canister-id">
                <a href={dashboardUrl(entry.canister_id)} target="_blank" rel="noopener">
                  {shortenCanisterId(entry.canister_id)}
                </a>
              </td>
              <td class="project" class:empty={!entry.project?.[0]}>
                {entry.project?.[0] ?? "Unknown"}
              </td>
              <td class="cycles">{formatCycles(entry.balance)}</td>
              <td class="burn {formatBurn(entry.burn_1h).class}">{formatBurn(entry.burn_1h).text}</td>
              <td class="burn {formatBurn(entry.burn_24h).class}">{formatBurn(entry.burn_24h).text}</td>
              <td class="burn {formatBurn(entry.burn_7d).class}">{formatBurn(entry.burn_7d).text}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}

  <footer>
    An <a href="https://alexandriadao.com/" target="_blank" rel="noopener">Alexandria</a> Project
  </footer>
</div>
