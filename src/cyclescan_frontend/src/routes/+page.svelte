<script>
  import "../index.scss";
  import { onMount } from "svelte";
  import { loadData, getProjectCanisters as fetchProjectCanisters, getProjectSparklineIntervals, getChartIntervals } from "$lib/data";
  import CanisterDetailModal from "$lib/components/CanisterDetailModal.svelte";
  import Sparkline from "$lib/components/Sparkline.svelte";
  import DataFreshness from "$lib/components/DataFreshness/DataFreshness.svelte";

  let entries = [];
  let projectEntries = [];
  let stats = null;
  let rawSnapshots = [];
  let loading = true;
  let error = null;
  let searchQuery = "";
  let sortColumn = "short_term_rate";
  let sortDirection = "desc";
  let currentPage = 1;
  let selectedCanisterId = null;
  let expandedProjects = new Set();
  let projectCanistersCache = new Map();
  let loadingProjects = new Set();
  let failedLogos = new Set();
  let includeCycleTransfers = false;

  // Sparkline caches (computed on demand for visible rows)
  let projectSparklineCache = new Map();
  let canisterSparklineCache = new Map();

  // Network-level stats
  let networkBurn24h = null;
  let networkBurnLoading = true;
  let xdrToUsd = 1.35;

  const ITEMS_PER_PAGE = 100;
  const TRILLION = 1_000_000_000_000n;
  const BILLION = 1_000_000_000n;
  const MILLION = 1_000_000n;
  const SECONDS_PER_DAY = 86400;
  const DAY_MS = 24 * 60 * 60 * 1000;

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

  // Format rate: takes cycles/hour, displays as cycles/day
  function formatRate(ratePerHour) {
    if (ratePerHour === null || ratePerHour === undefined) return null;
    const perDay = ratePerHour * 24n;

    const isNegative = perDay < 0n;
    const absPerDay = isNegative ? -perDay : perDay;

    let formatted;
    if (absPerDay >= TRILLION) {
      formatted = (Number(absPerDay / BILLION) / 1000).toFixed(2) + "T";
    } else if (absPerDay >= BILLION) {
      formatted = (Number(absPerDay / MILLION) / 1000).toFixed(2) + "B";
    } else if (absPerDay >= MILLION) {
      formatted = (Number(absPerDay) / 1_000_000).toFixed(1) + "M";
    } else {
      formatted = Number(absPerDay).toLocaleString();
    }

    return { value: formatted, isNegative };
  }

  // Format rate data for display in table - simplified
  function formatRateCell(rateData) {
    if (!rateData) {
      return { text: "-", class: "no-data" };
    }

    const formatted = formatRate(rateData.rate);
    if (!formatted) {
      return { text: "-", class: "no-data" };
    }

    let displayClass = "positive";
    if (formatted.isNegative) {
      displayClass = "gaining";
    } else if (rateData.rate === 0n) {
      displayClass = "zero";
    }

    return {
      text: formatted.isNegative ? `+${formatted.value}` : formatted.value,
      class: displayClass,
      dataPoints: rateData.dataPoints,
      hasInferred: rateData.hasInferredData
    };
  }

  // Calculate runway in days
  function calcRunway(balance, rateData) {
    if (!balance || !rateData || !rateData.rate || rateData.rate <= 0n) {
      return null;
    }
    const balanceBigInt = BigInt(balance);
    const ratePerDay = rateData.rate * 24n;
    if (ratePerDay <= 0n) return null;
    return Number(balanceBigInt / ratePerDay);
  }

  // Format runway for display
  function formatRunway(days) {
    if (days === null) {
      return { text: "∞", class: "runway-infinite" };
    }
    if (days < 30) {
      return { text: `${Math.round(days)}d`, class: "runway-critical" };
    } else if (days < 90) {
      return { text: `${Math.round(days)}d`, class: "runway-warning" };
    } else if (days < 365) {
      return { text: `${Math.round(days / 30)}mo`, class: "runway-ok" };
    } else {
      const years = days / 365;
      return { text: years >= 10 ? `${Math.round(years)}y` : `${years.toFixed(1)}y`, class: "runway-good" };
    }
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
    failedLogos = failedLogos;
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

  $: {
    searchQuery;
    sortColumn;
    sortDirection;
    currentPage = 1;
  }
  $: startIndex = (currentPage - 1) * ITEMS_PER_PAGE;

  // Get rate value for sorting
  function getRateValue(entry, col) {
    switch (col) {
      case "total_balance":
      case "balance":
        return entry.adj_total_balance ?? entry.total_balance ?? entry.balance ?? 0n;
      case "recent_rate":
        return entry.recent_rate?.rate ?? entry.adj_recent_rate?.rate ?? -1n;
      case "short_term_rate":
        return entry.short_term_rate?.rate ?? entry.adj_short_term_rate?.rate ?? -1n;
      case "long_term_rate":
        return entry.long_term_rate?.rate ?? entry.adj_long_term_rate?.rate ?? -1n;
      case "runway": {
        const balance = entry.adj_total_balance ?? entry.total_balance ?? entry.balance;
        const rate = entry.adj_short_term_rate ?? entry.short_term_rate;
        const days = calcRunway(balance, rate);
        return days === null ? Infinity : days;
      }
      case "project":
        return entry.project ?? "";
      case "canister_count":
        return entry.adj_canister_count ?? entry.canister_count ?? 0n;
      default:
        return 0n;
    }
  }

  $: filteredProjectEntries = adjustedProjectEntries.filter(e => {
    if (!searchQuery) return true;
    const q = searchQuery.toLowerCase();
    return e.project.toLowerCase().includes(q);
  });

  $: sortedProjectEntries = [...filteredProjectEntries].sort((a, b) => {
    const aVal = getRateValue(a, sortColumn);
    const bVal = getRateValue(b, sortColumn);
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

  function goToPage(page) {
    if (page >= 1 && page <= totalProjectPages) {
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
      expandedProjects = expandedProjects;
    } else {
      expandedProjects.add(projectName);
      expandedProjects = expandedProjects;

      if (!projectCanistersCache.has(projectName)) {
        loadingProjects.add(projectName);
        loadingProjects = loadingProjects;
        try {
          const canisters = await fetchProjectCanisters(projectName);
          projectCanistersCache.set(projectName, canisters);
          projectCanistersCache = projectCanistersCache;
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

  function getVisibleProjectCanisters(projectName) {
    const canisters = getProjectCanisters(projectName);
    if (includeCycleTransfers) return canisters;
    return canisters.filter(c => c.valid);
  }

  // Get sparkline data for a project (with caching)
  function getProjectSparklineData(projectName) {
    if (!projectSparklineCache.has(projectName)) {
      projectSparklineCache.set(projectName, getProjectSparklineIntervals(projectName, 7 * DAY_MS));
    }
    return projectSparklineCache.get(projectName);
  }

  // Get sparkline data for a canister (with caching)
  function getCanisterSparklineData(canisterId) {
    if (!canisterSparklineCache.has(canisterId)) {
      canisterSparklineCache.set(canisterId, getChartIntervals(canisterId, 7 * DAY_MS));
    }
    return canisterSparklineCache.get(canisterId);
  }

  // Pre-compute adjusted project entries (excluding invalid canisters)
  $: adjustedProjectEntries = (() => {
    const contrib = new Map();
    for (const entry of entries) {
      const project = entry.project?.[0];
      if (!project) continue;
      if (!contrib.has(project)) {
        contrib.set(project, {
          total: 0,
          invalid: 0,
          invalidBalance: 0n,
          invalidRecentRate: 0n,
          invalidShortTermRate: 0n,
          invalidLongTermRate: 0n
        });
      }
      const c = contrib.get(project);
      c.total++;
      if (!entry.valid) {
        c.invalid++;
        c.invalidBalance += BigInt(entry.balance || 0);
        c.invalidRecentRate += entry.recent_rate?.rate ?? 0n;
        c.invalidShortTermRate += entry.short_term_rate?.rate ?? 0n;
        c.invalidLongTermRate += entry.long_term_rate?.rate ?? 0n;
      }
    }

    return projectEntries.map(entry => {
      const c = contrib.get(entry.project);
      if (includeCycleTransfers || !c || c.invalid === 0) {
        return {
          ...entry,
          adj_canister_count: entry.canister_count,
          adj_total_balance: entry.total_balance,
          adj_recent_rate: entry.recent_rate,
          adj_short_term_rate: entry.short_term_rate,
          adj_long_term_rate: entry.long_term_rate
        };
      }

      const adjBalance = BigInt(entry.total_balance) - c.invalidBalance;

      const adjRecentRate = entry.recent_rate ? {
        ...entry.recent_rate,
        rate: entry.recent_rate.rate - c.invalidRecentRate
      } : null;
      const adjShortTermRate = entry.short_term_rate ? {
        ...entry.short_term_rate,
        rate: entry.short_term_rate.rate - c.invalidShortTermRate
      } : null;
      const adjLongTermRate = entry.long_term_rate ? {
        ...entry.long_term_rate,
        rate: entry.long_term_rate.rate - c.invalidLongTermRate
      } : null;

      return {
        ...entry,
        adj_canister_count: BigInt(entry.canister_count) - BigInt(c.invalid),
        adj_total_balance: adjBalance > 0n ? adjBalance : 0n,
        adj_recent_rate: adjRecentRate,
        adj_short_term_rate: adjShortTermRate,
        adj_long_term_rate: adjLongTermRate
      };
    });
  })();

  $: invalidCanisterCount = entries.filter(e => !e.valid).length;

  // Calculate aggregate burn from tracked canisters
  $: trackedBurn24h = (() => {
    const validEntries = entries.filter(e => includeCycleTransfers || e.valid);
    const totalRatePerHour = validEntries.reduce((sum, entry) => {
      const rate = entry.short_term_rate?.rate;
      if (rate !== null && rate !== undefined) {
        return sum + rate;
      }
      return sum;
    }, 0n);
    return totalRatePerHour * 24n;
  })();

  $: coveragePercent = (networkBurn24h && trackedBurn24h > 0n)
    ? Number((trackedBurn24h * 10000n) / BigInt(Math.floor(networkBurn24h))) / 100
    : null;

  async function fetchNetworkBurnRate() {
    try {
      const response = await fetch('https://ic-api.internetcomputer.org/api/v3/metrics/cycle-burn-rate');
      const data = await response.json();

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

  function cyclesToUsd(cycles) {
    if (cycles === null || cycles === undefined) return null;
    const n = typeof cycles === 'bigint' ? Number(cycles) : cycles;
    const trillions = n / 1e12;
    return trillions * xdrToUsd;
  }

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
    fetchNetworkBurnRate();

    try {
      const data = await loadData();
      entries = data.entries;
      stats = data.stats;
      projectEntries = data.projectEntries;
      rawSnapshots = data.rawSnapshots;
      loading = false;
    } catch (e) {
      error = e.message || "Failed to load data";
      loading = false;
    }
  });
</script>

<div class="container">
  <header class="page-header">
    <div class="header-left">
      <div class="header-row-top">
        <div class="header-brand" title="Cycles burn leaderboard for ICP">
          <img src="/cyclescan_canister.png" alt="CycleScan" class="header-logo" />
          <span class="brand-name">CycleScan</span>
        </div>
        <div class="meta-stats">
          <span class="meta-item" title="Number of canisters being tracked">
            {stats ? formatNumber(stats.canister_count) : '—'} canisters
          </span>
          <span class="meta-sep">·</span>
          <span class="meta-item" title={`Coverage: ${coveragePercent?.toFixed(1) ?? '—'}% of total IC network cycle burn is tracked by CycleScan`}>
            {#if loading || networkBurnLoading}—{:else if coveragePercent !== null}{coveragePercent.toFixed(1)}% coverage{:else}—{/if}
          </span>
          <span class="meta-sep">·</span>
          <span class="meta-item highlight" title={`Tracked burn: ${formatCycles(trackedBurn24h)}/day (${formatUsd(cyclesToUsd(trackedBurn24h))} USD at 1T cycles = $${xdrToUsd.toFixed(2)})`}>
            {#if loading}—{:else}{formatUsd(cyclesToUsd(trackedBurn24h))}/day burn{/if}
          </span>
          <span class="meta-sep">·</span>
          <a href="/about" class="meta-link" title="How it works">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"></circle>
              <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path>
              <line x1="12" y1="17" x2="12.01" y2="17"></line>
            </svg>
          </a>
        </div>
      </div>
      <div class="header-row-bottom">
        <input
          type="text"
          class="search"
          placeholder="Search projects..."
          bind:value={searchQuery}
        />
        <label class="toggle-label">
          <input type="checkbox" bind:checked={includeCycleTransfers} />
          <span>Include cycle transfers</span>
          {#if invalidCanisterCount > 0 && !includeCycleTransfers}
            <span class="excluded-count" title="Canisters excluded because they transfer cycles rather than burn them">
              ({invalidCanisterCount} excluded)
            </span>
          {/if}
        </label>
      </div>
    </div>
    {#if !loading && rawSnapshots.length > 0}
      <div class="header-right">
        <DataFreshness snapshots={rawSnapshots} />
      </div>
    {/if}
  </header>

  {#if loading}
    <div class="loading">Loading leaderboard...</div>
  {:else if error}
    <div class="error">Error: {error}</div>
  {:else}
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
              <th
                class="num-col"
                class:sorted={sortColumn === "canister_count"}
                on:click={() => sortBy("canister_count")}
                title="Canisters"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path>
                </svg>
                <span class="sort-arrow">{sortColumn === "canister_count" ? (sortDirection === "desc" ? "▼" : "▲") : "▼"}</span>
              </th>
              <th
                class="col-balance"
                class:sorted={sortColumn === "balance"}
                on:click={() => sortBy("balance")}
              >
                Balance
                <span class="sort-arrow">{sortColumn === "balance" ? (sortDirection === "desc" ? "▼" : "▲") : "▼"}</span>
              </th>
              <th
                class="col-burn"
                class:sorted={sortColumn === "recent_rate"}
                on:click={() => sortBy("recent_rate")}
                title="Burn rate from ~2 hours of data"
              >
                Recent
                <span class="time-hint">(~2h)</span>
                <span class="sort-arrow">{sortColumn === "recent_rate" ? (sortDirection === "desc" ? "▼" : "▲") : "▼"}</span>
              </th>
              <th
                class="col-burn"
                class:sorted={sortColumn === "short_term_rate"}
                on:click={() => sortBy("short_term_rate")}
                title="Burn rate from ~36 hours of data"
              >
                Short-term
                <span class="time-hint">(~24h)</span>
                <span class="sort-arrow">{sortColumn === "short_term_rate" ? (sortDirection === "desc" ? "▼" : "▲") : "▼"}</span>
              </th>
              <th
                class="col-burn"
                class:sorted={sortColumn === "long_term_rate"}
                on:click={() => sortBy("long_term_rate")}
                title="Burn rate from ~7 days of data"
              >
                Long-term
                <span class="time-hint">(~7d)</span>
                <span class="sort-arrow">{sortColumn === "long_term_rate" ? (sortDirection === "desc" ? "▼" : "▲") : "▼"}</span>
              </th>
              <th
                class="col-runway"
                class:sorted={sortColumn === "runway"}
                on:click={() => sortBy("runway")}
                title="Estimated days until cycles depleted"
              >
                Runway
                <span class="sort-arrow">{sortColumn === "runway" ? (sortDirection === "desc" ? "▼" : "▲") : "▼"}</span>
              </th>
              <th class="col-trend">
                Trend
                <span class="time-hint">(24h)</span>
              </th>
            </tr>
          </thead>
          <tbody>
            {#each paginatedProjectEntries as entry, i}
              {@const recentCell = formatRateCell(entry.adj_recent_rate ?? entry.recent_rate)}
              {@const shortTermCell = formatRateCell(entry.adj_short_term_rate ?? entry.short_term_rate)}
              {@const longTermCell = formatRateCell(entry.adj_long_term_rate ?? entry.long_term_rate)}
              {@const runwayDays = calcRunway(entry.adj_total_balance ?? entry.total_balance, entry.adj_short_term_rate ?? entry.short_term_rate)}
              {@const runwayCell = formatRunway(runwayDays)}
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
                    {#if entry.website?.[0]}
                      <a href={entry.website[0]} target="_blank" rel="noopener noreferrer" class="website-link-inline" on:click|stopPropagation title={entry.website[0]}>
                        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                          <circle cx="12" cy="12" r="10"></circle>
                          <line x1="2" y1="12" x2="22" y2="12"></line>
                          <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
                        </svg>
                      </a>
                    {/if}
                  </div>
                </td>
                <td class="canister-count">{Number(entry.adj_canister_count ?? entry.canister_count).toLocaleString()}</td>
                <td class="cycles">{formatCycles(entry.adj_total_balance ?? entry.total_balance)}</td>
                <td class="burn {recentCell.class}">
                  <span class="rate-value" title={recentCell.dataPoints ? `${recentCell.dataPoints} data points` : ''}>
                    {recentCell.text}
                    {#if recentCell.text !== "-"}<span class="rate-suffix">/day</span>{/if}
                  </span>
                </td>
                <td class="burn {shortTermCell.class}">
                  <span class="rate-value" title={shortTermCell.dataPoints ? `${shortTermCell.dataPoints} data points` : ''}>
                    {shortTermCell.text}
                    {#if shortTermCell.text !== "-"}<span class="rate-suffix">/day</span>{/if}
                  </span>
                </td>
                <td class="burn {longTermCell.class}">
                  <span class="rate-value" title={longTermCell.dataPoints ? `${longTermCell.dataPoints} data points` : ''}>
                    {longTermCell.text}
                    {#if longTermCell.text !== "-"}<span class="rate-suffix">/day</span>{/if}
                  </span>
                </td>
                <td class="runway {runwayCell.class}" title={runwayDays !== null ? `${Math.round(runwayDays)} days` : 'Infinite'}>
                  {runwayCell.text}
                </td>
                <td class="trend">
                  <Sparkline intervals={getProjectSparklineData(entry.project)} showInferred={false} />
                </td>
              </tr>
              {#if expandedProjects.has(entry.project)}
                {#if loadingProjects.has(entry.project)}
                  <tr class="sub-row loading-row">
                    <td colspan="9" class="loading-cell">Loading canisters...</td>
                  </tr>
                {:else}
                  {#each getVisibleProjectCanisters(entry.project) as canister, j}
                    {@const canRecentCell = formatRateCell(canister.recent_rate)}
                    {@const canShortTermCell = formatRateCell(canister.short_term_rate)}
                    {@const canLongTermCell = formatRateCell(canister.long_term_rate)}
                    {@const canRunwayDays = calcRunway(canister.balance, canister.short_term_rate)}
                    {@const canRunwayCell = formatRunway(canRunwayDays)}
                    <tr class="sub-row clickable" on:click|stopPropagation={() => openModal(canister.canister_id)}>
                      <td class="rank sub-rank"></td>
                      <td class="project sub-project">
                        <div class="project-cell sub-cell">
                          <span class="sub-canister-id">{shortenCanisterId(canister.canister_id)}</span>
                          {#if !canister.valid}
                            <span class="transfers-flag" title="This canister transfers cycles rather than burns them">
                              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M4 15s1-1 4-1 5 2 8 2 4-1 4-1V3s-1 1-4 1-5-2-8-2-4 1-4 1z"></path>
                                <line x1="4" y1="22" x2="4" y2="15" stroke="currentColor" stroke-width="2"></line>
                              </svg>
                            </span>
                          {/if}
                        </div>
                      </td>
                      <td class="canister-count"></td>
                      <td class="cycles">{formatCycles(canister.balance)}</td>
                      <td class="burn {canRecentCell.class}">
                        <span class="rate-value">
                          {canRecentCell.text}
                          {#if canRecentCell.text !== "-"}<span class="rate-suffix">/day</span>{/if}
                        </span>
                      </td>
                      <td class="burn {canShortTermCell.class}">
                        <span class="rate-value">
                          {canShortTermCell.text}
                          {#if canShortTermCell.text !== "-"}<span class="rate-suffix">/day</span>{/if}
                        </span>
                      </td>
                      <td class="burn {canLongTermCell.class}">
                        <span class="rate-value">
                          {canLongTermCell.text}
                          {#if canLongTermCell.text !== "-"}<span class="rate-suffix">/day</span>{/if}
                        </span>
                      </td>
                      <td class="runway {canRunwayCell.class}">
                        {canRunwayCell.text}
                      </td>
                      <td class="trend sub-trend">
                        <Sparkline intervals={getCanisterSparklineData(canister.canister_id)} width={60} height={16} />
                      </td>
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

  <!-- Pagination -->
  {#if !loading && !error && totalProjectPages > 1}
    <div class="pagination">
      <button class="page-btn" disabled={currentPage === 1} on:click={() => goToPage(1)}>First</button>
      <button class="page-btn" disabled={currentPage === 1} on:click={() => goToPage(currentPage - 1)}>Prev</button>
      <div class="page-numbers">
        {#each Array.from({ length: totalProjectPages }, (_, i) => i + 1) as page}
          {#if page === 1 || page === totalProjectPages || (page >= currentPage - 2 && page <= currentPage + 2)}
            <button class="page-num" class:active={page === currentPage} on:click={() => goToPage(page)}>{page}</button>
          {:else if page === currentPage - 3 || page === currentPage + 3}
            <span class="ellipsis">...</span>
          {/if}
        {/each}
      </div>
      <button class="page-btn" disabled={currentPage === totalProjectPages} on:click={() => goToPage(currentPage + 1)}>Next</button>
      <button class="page-btn" disabled={currentPage === totalProjectPages} on:click={() => goToPage(totalProjectPages)}>Last</button>
      <span class="page-info">{startIndex + 1}-{Math.min(startIndex + ITEMS_PER_PAGE, sortedProjectEntries.length)} of {sortedProjectEntries.length.toLocaleString()}</span>
    </div>
  {/if}

  <footer>
    <a href="/about">How It Works</a>
    <span class="meta-sep">·</span>
    An <a href="https://alexandriadao.com/" target="_blank" rel="noopener">Alexandria</a> Project
  </footer>
</div>

{#if selectedCanisterId}
  <CanisterDetailModal canisterId={selectedCanisterId} onClose={closeModal} />
{/if}
