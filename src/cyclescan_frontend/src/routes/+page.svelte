<script>
  import "../index.scss";
  import { onMount } from "svelte";
  import { backend } from "$lib/canisters";
  import CanisterDetailModal from "$lib/components/CanisterDetailModal.svelte";

  let entries = [];
  let projectEntries = [];
  let stats = null;
  let loading = true;
  let projectLoading = false;
  let error = null;
  let searchQuery = "";
  let sortColumn = "burn_24h";
  let sortDirection = "desc";
  let currentPage = 1;
  let selectedCanisterId = null;
  let viewMode = "canisters"; // "canisters" or "projects"
  let expandedProjects = new Set(); // Track which projects are expanded
  let projectCanistersCache = new Map(); // Cache for project canisters
  let loadingProjects = new Set(); // Track which projects are loading
  let failedLogos = new Set(); // Track logos that failed to load

  // Network-level stats
  let networkBurn24h = null;
  let networkBurnLoading = true;
  let xdrToUsd = 1.35; // 1 XDR ≈ $1.35 USD, 1 trillion cycles = 1 XDR

  const ITEMS_PER_PAGE = 100;
  const TRILLION = 1_000_000_000_000n;
  const BILLION = 1_000_000_000n;
  const MILLION = 1_000_000n;
  const SECONDS_PER_DAY = 86400;

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

  function getLogoPath(project) {
    if (!project) return null;
    const filename = project.toLowerCase().replace(/[^a-z0-9]/g, '-').replace(/-+/g, '-').replace(/^-|-$/g, '');
    return `/logos/${filename}.png`;
  }

  function handleLogoError(project) {
    failedLogos.add(project);
    failedLogos = failedLogos; // Trigger reactivity
  }

  async function copyToClipboard(text, event) {
    event.stopPropagation();
    try {
      await navigator.clipboard.writeText(text);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
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

  $: totalPages = Math.ceil(sortedEntries.length / ITEMS_PER_PAGE);
  $: {
    // Reset to page 1 when filters change
    searchQuery;
    sortColumn;
    sortDirection;
    currentPage = 1;
  }
  $: startIndex = (currentPage - 1) * ITEMS_PER_PAGE;
  $: paginatedEntries = sortedEntries.slice(startIndex, startIndex + ITEMS_PER_PAGE);

  // Project view helpers
  function getProjectValue(entry, col) {
    switch (col) {
      case "total_balance": return entry.total_balance;
      case "total_burn_1h": return entry.total_burn_1h?.[0] ?? -1n;
      case "total_burn_24h": return entry.total_burn_24h?.[0] ?? -1n;
      case "total_burn_7d": return entry.total_burn_7d?.[0] ?? -1n;
      case "project": return entry.project;
      case "canister_count": return entry.canister_count;
      default: return 0;
    }
  }

  $: filteredProjectEntries = projectEntries.filter(e => {
    if (!searchQuery) return true;
    const q = searchQuery.toLowerCase();
    return e.project.toLowerCase().includes(q);
  });

  $: sortedProjectEntries = [...filteredProjectEntries].sort((a, b) => {
    // Map canister columns to project columns for sorting
    let col = sortColumn;
    if (col === "balance") col = "total_balance";
    if (col === "burn_1h") col = "total_burn_1h";
    if (col === "burn_24h") col = "total_burn_24h";
    if (col === "burn_7d") col = "total_burn_7d";

    const aVal = getProjectValue(a, col);
    const bVal = getProjectValue(b, col);
    let cmp = 0;
    if (typeof aVal === "string") {
      cmp = aVal.localeCompare(bVal);
    } else {
      if (aVal < bVal) cmp = -1;
      else if (aVal > bVal) cmp = 1;
    }
    return sortDirection === "desc" ? -cmp : cmp;
  });

  $: totalProjectPages = Math.ceil(sortedProjectEntries.length / ITEMS_PER_PAGE);
  $: paginatedProjectEntries = sortedProjectEntries.slice(startIndex, startIndex + ITEMS_PER_PAGE);

  // Use correct totals based on view
  $: currentTotalPages = viewMode === "projects" ? totalProjectPages : totalPages;
  $: currentTotalEntries = viewMode === "projects" ? sortedProjectEntries.length : sortedEntries.length;

  function goToPage(page) {
    if (page >= 1 && page <= currentTotalPages) {
      currentPage = page;
    }
  }

  function formatNumber(n) {
    return Number(n).toLocaleString();
  }

  function openModal(canisterId) {
    selectedCanisterId = canisterId.toString();
  }

  function closeModal() {
    selectedCanisterId = null;
  }

  async function toggleProjectExpanded(projectName) {
    if (expandedProjects.has(projectName)) {
      expandedProjects.delete(projectName);
      expandedProjects = expandedProjects; // Trigger reactivity
    } else {
      expandedProjects.add(projectName);
      expandedProjects = expandedProjects; // Trigger reactivity

      // Fetch canisters if not already cached
      if (!projectCanistersCache.has(projectName)) {
        loadingProjects.add(projectName);
        loadingProjects = loadingProjects;
        try {
          const canisters = await backend.get_project_canisters(projectName);
          projectCanistersCache.set(projectName, canisters);
          projectCanistersCache = projectCanistersCache; // Trigger reactivity
        } catch (e) {
          console.error(`Failed to fetch canisters for ${projectName}:`, e);
        } finally {
          loadingProjects.delete(projectName);
          loadingProjects = loadingProjects;
        }
      }
    }
  }

  function getProjectCanisters(projectName) {
    return projectCanistersCache.get(projectName) || [];
  }

  // Calculate aggregate 24hr burn from tracked canisters
  $: trackedBurn24h = entries.reduce((sum, entry) => {
    const burn = entry.burn_24h?.[0];
    if (burn !== null && burn !== undefined) {
      return sum + BigInt(burn);
    }
    return sum;
  }, 0n);

  // Calculate coverage percentage
  $: coveragePercent = (networkBurn24h && trackedBurn24h > 0n)
    ? Number((trackedBurn24h * 10000n) / BigInt(Math.floor(networkBurn24h))) / 100
    : null;

  // Get top 3 burners for highlights
  $: topBurners = entries
    .filter(e => e.burn_24h?.[0] && e.burn_24h[0] > 0n)
    .sort((a, b) => {
      const aVal = a.burn_24h?.[0] ?? 0n;
      const bVal = b.burn_24h?.[0] ?? 0n;
      if (aVal < bVal) return 1;
      if (aVal > bVal) return -1;
      return 0;
    })
    .slice(0, 3);

  // Fetch network-wide cycle burn rate from IC Dashboard API
  async function fetchNetworkBurnRate() {
    try {
      const response = await fetch('https://ic-api.internetcomputer.org/api/v3/metrics/cycle-burn-rate');
      const data = await response.json();

      // API returns cycles per second, multiply by seconds per day
      if (data.cycle_burn_rate && data.cycle_burn_rate.length > 0) {
        const latestRate = parseFloat(data.cycle_burn_rate[data.cycle_burn_rate.length - 1][1]);
        networkBurn24h = latestRate * SECONDS_PER_DAY;
      }
    } catch (e) {
      console.error('Failed to fetch network burn rate:', e);
    } finally {
      networkBurnLoading = false;
    }
  }

  // Fetch project leaderboard
  async function fetchProjectLeaderboard() {
    if (projectEntries.length > 0) return; // Already fetched
    projectLoading = true;
    try {
      projectEntries = await backend.get_project_leaderboard();
    } catch (e) {
      console.error('Failed to fetch project leaderboard:', e);
    } finally {
      projectLoading = false;
    }
  }

  // Handle view mode change
  async function setViewMode(mode) {
    viewMode = mode;
    searchQuery = ""; // Reset search when switching views
    currentPage = 1;
    if (mode === "projects") {
      await fetchProjectLeaderboard();
    }
  }

  // Format cycles in Trillions for hero display (consistent unit for comparison)
  function formatTrillions(value) {
    if (value === null || value === undefined) return null;
    const n = typeof value === 'bigint' ? Number(value) : value;
    return (n / 1e12).toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 0 });
  }

  // Convert cycles to USD (1 trillion cycles = 1 XDR ≈ $1.35)
  function cyclesToUsd(cycles) {
    if (cycles === null || cycles === undefined) return null;
    const n = typeof cycles === 'bigint' ? Number(cycles) : cycles;
    const trillions = n / 1e12;
    return trillions * xdrToUsd;
  }

  // Format USD for display
  function formatUsd(usd) {
    if (usd === null || usd === undefined) return '-';
    if (usd >= 1000000) {
      return '$' + (usd / 1000000).toFixed(2) + 'M';
    } else if (usd >= 1000) {
      return '$' + (usd / 1000).toFixed(1) + 'K';
    } else {
      return '$' + usd.toFixed(0);
    }
  }

  onMount(async () => {
    // Fetch network burn rate in parallel (don't block main data)
    fetchNetworkBurnRate();

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
  <div class="stats-bar">
    <div class="stat-item">
      <span class="stat-label">Network Burn:</span>
      <span class="stat-value network">
        {#if networkBurnLoading}...{:else}{formatUsd(cyclesToUsd(networkBurn24h))}/day{/if}
      </span>
    </div>
    <div class="stat-item">
      <span class="stat-label">Tracked:</span>
      <span class="stat-value tracked">
        {#if loading}...{:else}{formatUsd(cyclesToUsd(trackedBurn24h))}/day{/if}
      </span>
      <span class="stat-secondary">({formatTrillions(trackedBurn24h)}T)</span>
    </div>
    <div class="stat-item">
      <span class="stat-label">Coverage:</span>
      <span class="stat-value coverage">
        {#if loading || networkBurnLoading}...{:else if coveragePercent !== null}{coveragePercent.toFixed(1)}%{:else}-{/if}
      </span>
    </div>
    <div class="stat-item">
      <span class="stat-label">Canisters:</span>
      <span class="stat-value">{stats ? formatNumber(stats.canister_count) : '-'}</span>
    </div>
  </div>

  <header class="page-header">
    <div class="header-main">
      <div class="header-title">
        <div class="brand-row">
          <img src="/logo.png" alt="CycleScan" class="header-logo" />
          <div>
            <h1>CycleScan</h1>
            <p class="tagline">Cycle consumption leaderboard for the Internet Computer. See which canisters burn the most compute.</p>
          </div>
        </div>
      </div>
      {#if !loading && topBurners.length > 0}
        <div class="top-burners">
          <span class="burners-label">Top Burners</span>
          {#each topBurners as burner, i}
            <button class="burner-chip" on:click={() => openModal(burner.canister_id)}>
              <span class="burner-rank">#{i + 1}</span>
              <span class="burner-name">{burner.project?.[0] ?? shortenCanisterId(burner.canister_id)}</span>
              <span class="burner-amount">{formatUsd(cyclesToUsd(burner.burn_24h[0]))}</span>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  </header>

  <div class="controls">
    <div class="view-toggle">
      <button
        class="toggle-btn"
        class:active={viewMode === "canisters"}
        on:click={() => setViewMode("canisters")}
      >
        By Canister
      </button>
      <button
        class="toggle-btn"
        class:active={viewMode === "projects"}
        on:click={() => setViewMode("projects")}
      >
        By Project
      </button>
    </div>
    <input
      type="text"
      class="search"
      placeholder={viewMode === "projects" ? "Search by project..." : "Search by canister ID or project..."}
      bind:value={searchQuery}
    />
  </div>

  {#if loading || (viewMode === "projects" && projectLoading)}
    <div class="loading">Loading leaderboard...</div>
  {:else if error}
    <div class="error">Error: {error}</div>
  {:else if viewMode === "canisters"}
    <!-- Canister View -->
    {#if sortedEntries.length === 0}
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
                class:sorted={sortColumn === "project"}
                on:click={() => sortBy("project")}
              >
                Project
                <span class="sort-arrow">{sortColumn === "project" ? (sortDirection === "desc" ? "▼" : "▲") : "▼"}</span>
              </th>
              <th class="website-col">Web</th>
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
              <th
                class:sorted={sortColumn === "canister_id"}
                on:click={() => sortBy("canister_id")}
              >
                Canister
                <span class="sort-arrow">{sortColumn === "canister_id" ? (sortDirection === "desc" ? "▼" : "▲") : "▼"}</span>
              </th>
            </tr>
          </thead>
          <tbody>
            {#each paginatedEntries as entry, i}
              <tr class="clickable" on:click={() => openModal(entry.canister_id)}>
                <td class="rank">{startIndex + i + 1}</td>
                <td class="project" class:empty={!entry.project?.[0]}>
                  <div class="project-cell">
                    {#if entry.project?.[0] && !failedLogos.has(entry.project[0])}
                      <img
                        src={getLogoPath(entry.project[0])}
                        alt=""
                        class="project-logo"
                        on:error={() => handleLogoError(entry.project[0])}
                      />
                    {/if}
                    <span class="project-name">{entry.project?.[0] ?? "Unknown"}</span>
                  </div>
                </td>
                <td class="website-cell">
                  {#if entry.website?.[0]}
                    <a href={entry.website[0]} target="_blank" rel="noopener noreferrer" class="website-link" on:click|stopPropagation title={entry.website[0]}>
                      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="12" cy="12" r="10"></circle>
                        <line x1="2" y1="12" x2="22" y2="12"></line>
                        <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
                      </svg>
                    </a>
                  {/if}
                </td>
                <td class="cycles">{formatCycles(entry.balance)}</td>
                <td class="burn {formatBurn(entry.burn_1h).class}">{formatBurn(entry.burn_1h).text}</td>
                <td class="burn {formatBurn(entry.burn_24h).class}">{formatBurn(entry.burn_24h).text}</td>
                <td class="burn {formatBurn(entry.burn_7d).class}">{formatBurn(entry.burn_7d).text}</td>
                <td class="canister-id">
                  <span class="canister-link">
                    {shortenCanisterId(entry.canister_id)}
                  </span>
                  <button class="copy-btn" on:click={(e) => copyToClipboard(entry.canister_id.toString(), e)} title="Copy canister ID">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                      <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                    </svg>
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {:else}
    <!-- Project View -->
    {#if sortedProjectEntries.length === 0}
      <div class="empty-state">
        {#if searchQuery}
          No projects match your search.
        {:else}
          No projects with named canisters yet.
        {/if}
      </div>
    {:else}
      <div class="table-wrapper">
        <table>
          <thead>
            <tr>
              <th class="rank">#</th>
              <th
                class:sorted={sortColumn === "project"}
                on:click={() => sortBy("project")}
              >
                Project
                <span class="sort-arrow">{sortColumn === "project" ? (sortDirection === "desc" ? "▼" : "▲") : "▼"}</span>
              </th>
              <th class="website-col">Web</th>
              <th
                class:sorted={sortColumn === "canister_count"}
                on:click={() => sortBy("canister_count")}
              >
                Canisters
                <span class="sort-arrow">{sortColumn === "canister_count" ? (sortDirection === "desc" ? "▼" : "▲") : "▼"}</span>
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
            {#each paginatedProjectEntries as entry, i}
              <tr class="project-row clickable" class:expanded={expandedProjects.has(entry.project)} on:click={() => toggleProjectExpanded(entry.project)}>
                <td class="rank">{startIndex + i + 1}</td>
                <td class="project">
                  <div class="project-cell">
                    <span class="expand-icon" class:expanded={expandedProjects.has(entry.project)}>
                      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <polyline points="9 18 15 12 9 6"></polyline>
                      </svg>
                    </span>
                    {#if !failedLogos.has(entry.project)}
                      <img
                        src={getLogoPath(entry.project)}
                        alt=""
                        class="project-logo"
                        on:error={() => handleLogoError(entry.project)}
                      />
                    {/if}
                    <span class="project-name">{entry.project}</span>
                  </div>
                </td>
                <td class="website-cell">
                  {#if entry.website?.[0]}
                    <a href={entry.website[0]} target="_blank" rel="noopener noreferrer" class="website-link" on:click|stopPropagation title={entry.website[0]}>
                      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="12" cy="12" r="10"></circle>
                        <line x1="2" y1="12" x2="22" y2="12"></line>
                        <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
                      </svg>
                    </a>
                  {/if}
                </td>
                <td class="canister-count">{Number(entry.canister_count).toLocaleString()}</td>
                <td class="cycles">{formatCycles(entry.total_balance)}</td>
                <td class="burn {formatBurn(entry.total_burn_1h).class}">{formatBurn(entry.total_burn_1h).text}</td>
                <td class="burn {formatBurn(entry.total_burn_24h).class}">{formatBurn(entry.total_burn_24h).text}</td>
                <td class="burn {formatBurn(entry.total_burn_7d).class}">{formatBurn(entry.total_burn_7d).text}</td>
              </tr>
              {#if expandedProjects.has(entry.project)}
                {#if loadingProjects.has(entry.project)}
                  <tr class="sub-row loading-row">
                    <td colspan="8" class="loading-cell">Loading canisters...</td>
                  </tr>
                {:else}
                  {#each getProjectCanisters(entry.project) as canister, j}
                    <tr class="sub-row clickable" on:click|stopPropagation={() => openModal(canister.canister_id)}>
                      <td class="rank sub-rank"></td>
                      <td class="project sub-project">
                        <div class="project-cell sub-cell">
                          <span class="sub-canister-id">{shortenCanisterId(canister.canister_id)}</span>
                        </div>
                      </td>
                      <td class="website-cell"></td>
                      <td class="canister-count"></td>
                      <td class="cycles">{formatCycles(canister.balance)}</td>
                      <td class="burn {formatBurn(canister.burn_1h).class}">{formatBurn(canister.burn_1h).text}</td>
                      <td class="burn {formatBurn(canister.burn_24h).class}">{formatBurn(canister.burn_24h).text}</td>
                      <td class="burn {formatBurn(canister.burn_7d).class}">{formatBurn(canister.burn_7d).text}</td>
                    </tr>
                  {/each}
                {/if}
              {/if}
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {/if}

  <!-- Pagination (shared for both views) -->
  {#if !loading && !error && currentTotalPages > 1}
    <div class="pagination">
      <button
        class="page-btn"
        disabled={currentPage === 1}
        on:click={() => goToPage(1)}
      >
        First
      </button>
      <button
        class="page-btn"
        disabled={currentPage === 1}
        on:click={() => goToPage(currentPage - 1)}
      >
        Prev
      </button>

      <div class="page-numbers">
        {#each Array.from({ length: currentTotalPages }, (_, i) => i + 1) as page}
          {#if page === 1 || page === currentTotalPages || (page >= currentPage - 2 && page <= currentPage + 2)}
            <button
              class="page-num"
              class:active={page === currentPage}
              on:click={() => goToPage(page)}
            >
              {page}
            </button>
          {:else if page === currentPage - 3 || page === currentPage + 3}
            <span class="ellipsis">...</span>
          {/if}
        {/each}
      </div>

      <button
        class="page-btn"
        disabled={currentPage === currentTotalPages}
        on:click={() => goToPage(currentPage + 1)}
      >
        Next
      </button>
      <button
        class="page-btn"
        disabled={currentPage === currentTotalPages}
        on:click={() => goToPage(currentTotalPages)}
      >
        Last
      </button>

      <span class="page-info">
        {startIndex + 1}-{Math.min(startIndex + ITEMS_PER_PAGE, currentTotalEntries)} of {currentTotalEntries.toLocaleString()}
      </span>
    </div>
  {/if}

  <footer>
    An <a href="https://alexandriadao.com/" target="_blank" rel="noopener">Alexandria</a> Project
  </footer>
</div>

{#if selectedCanisterId}
  <CanisterDetailModal
    canisterId={selectedCanisterId}
    onClose={closeModal}
  />
{/if}
