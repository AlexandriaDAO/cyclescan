<script>
  import "../../index.scss";
</script>

<div class="container">
  <header class="page-header">
    <div class="header-brand">
      <a href="/" class="back-link">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="15 18 9 12 15 6"></polyline>
        </svg>
      </a>
      <img src="/logo.png" alt="CycleScan" class="header-logo" />
      <span class="brand-name">CycleScan</span>
    </div>
    <h1 class="page-title">How It Works</h1>
  </header>

  <div class="content">
    <section class="methodology-section">
      <h2>Data Collection</h2>
      <p>
        CycleScan collects cycle balance data <strong>hourly</strong> using GitHub Actions.
        Each hour, we query ~2,900 canisters across the Internet Computer to record their
        current cycle balance. This data is stored in a public JSON file and fetched directly
        by your browser.
      </p>
      <div class="info-box">
        <strong>Why hourly?</strong> Hourly collection provides enough data points for accurate
        trend analysis while keeping infrastructure costs minimal. We store 7 days of hourly
        snapshots (~168 data points per canister).
      </div>
    </section>

    <section class="methodology-section">
      <h2>Burn Rate Calculation</h2>
      <p>
        Unlike simple "subtract previous from current" approaches, CycleScan uses
        <strong>linear regression</strong> to calculate burn rates. This provides several advantages:
      </p>
      <ul>
        <li><strong>Noise reduction:</strong> Minor fluctuations in individual snapshots don't skew the result</li>
        <li><strong>Uses all data:</strong> Instead of just two points, we use all available snapshots in the time window</li>
        <li><strong>Confidence metric:</strong> The R² value tells you how well the data fits a linear trend</li>
      </ul>

      <h3>The Math</h3>
      <p>
        For each canister, we collect all balance snapshots within a time window and fit a line
        using least-squares regression. The slope of this line (in cycles per millisecond) is
        converted to cycles per day for display.
      </p>
      <div class="formula">
        Burn Rate = -slope × 86,400,000 ms/day
      </div>
      <p class="formula-note">
        A negative slope means the balance is decreasing (burning cycles). We negate it to
        show burn as a positive number.
      </p>
    </section>

    <section class="methodology-section">
      <h2>Time Windows</h2>
      <p>We calculate burn rates over three time windows:</p>
      <table class="info-table">
        <tr>
          <th>Window</th>
          <th>Data Used</th>
          <th>Best For</th>
        </tr>
        <tr>
          <td><strong>Recent</strong></td>
          <td>Last ~2 hours</td>
          <td>Detecting sudden changes in activity</td>
        </tr>
        <tr>
          <td><strong>Short-term</strong></td>
          <td>Last ~36 hours</td>
          <td>Day-over-day trends, primary metric</td>
        </tr>
        <tr>
          <td><strong>Long-term</strong></td>
          <td>Last ~7 days</td>
          <td>Stable baseline burn rate</td>
        </tr>
      </table>
    </section>

    <section class="methodology-section">
      <h2>Handling Top-Ups</h2>
      <p>
        When a canister receives new cycles (a "top-up"), its balance jumps upward. This can
        distort burn rate calculations. CycleScan detects top-ups (increases ≥500B cycles) and
        handles them intelligently:
      </p>
      <ul>
        <li>
          <strong>Single top-up:</strong> We use only the data <em>after</em> the top-up for
          the most accurate recent burn rate.
        </li>
        <li>
          <strong>Multiple top-ups:</strong> We calculate burn from non-top-up intervals only,
          then extrapolate.
        </li>
      </ul>

      <h3>The ~ Symbol</h3>
      <p>
        When you see a <span class="badge-example">~</span> next to a burn rate, it means we
        couldn't reliably compensate for top-ups in that time window. The rate shown is our
        best estimate but may be less accurate.
      </p>
    </section>

    <section class="methodology-section">
      <h2>Confidence (R²)</h2>
      <p>
        The R² (R-squared) value measures how well the data fits a linear trend:
      </p>
      <ul>
        <li><strong>R² ≥ 0.9:</strong> Excellent fit - very consistent burn rate</li>
        <li><strong>R² ≥ 0.7:</strong> Good fit - reasonably stable</li>
        <li><strong>R² ≥ 0.5:</strong> Moderate fit - some variance</li>
        <li><strong>R² &lt; 0.5:</strong> Low confidence - burn rate varies significantly</li>
      </ul>
      <p>
        Low confidence doesn't mean the data is wrong—it means the canister's burn rate
        isn't constant. This is common for canisters with bursty activity.
      </p>
    </section>

    <section class="methodology-section">
      <h2>Runway Calculation</h2>
      <p>
        Runway estimates how long until a canister runs out of cycles at the current burn rate:
      </p>
      <div class="formula">
        Runway = Current Balance ÷ (Burn Rate per Day)
      </div>
      <table class="info-table">
        <tr>
          <th>Display</th>
          <th>Meaning</th>
        </tr>
        <tr>
          <td class="runway-critical">&lt; 30d</td>
          <td>Critical - canister may stop soon</td>
        </tr>
        <tr>
          <td class="runway-warning">30-90d</td>
          <td>Warning - consider topping up</td>
        </tr>
        <tr>
          <td class="runway-ok">90d-1y</td>
          <td>Okay - healthy runway</td>
        </tr>
        <tr>
          <td class="runway-good">> 1y</td>
          <td>Good - well funded</td>
        </tr>
        <tr>
          <td class="runway-infinite">∞</td>
          <td>Infinite - not burning or gaining cycles</td>
        </tr>
      </table>
    </section>

    <section class="methodology-section">
      <h2>Cycle Transfer Detection</h2>
      <p>
        Some canisters transfer cycles to other canisters rather than burning them. This creates
        artificially high "burn" rates. CycleScan marks these canisters and excludes them from
        aggregates by default.
      </p>
      <p>
        You can toggle "Include cycle transfers" to see all canisters, but be aware that
        projects with cycle-transferring canisters may show inflated burn rates.
      </p>
    </section>

    <section class="methodology-section">
      <h2>Data Sources</h2>
      <ul>
        <li>
          <strong>Canister balances:</strong> Queried via the
          <a href="https://dashboard.internetcomputer.org/canister/e3mmv-5qaaa-aaaah-aadma-cai" target="_blank" rel="noopener">Blackhole canister</a>
          (for individual canisters) and SNS Root canisters (for SNS projects)
        </li>
        <li>
          <strong>Network-wide burn:</strong>
          <a href="https://ic-api.internetcomputer.org/api/v3/metrics/cycle-burn-rate" target="_blank" rel="noopener">IC API</a>
        </li>
        <li>
          <strong>Project metadata:</strong> Curated from public sources
        </li>
      </ul>
    </section>

    <section class="methodology-section">
      <h2>Limitations</h2>
      <ul>
        <li>
          <strong>Collection timing:</strong> GitHub Actions may have slight variations in when
          they run, so "hourly" is approximate (±15 minutes typical).
        </li>
        <li>
          <strong>Coverage:</strong> We track ~2,900 canisters, which is a fraction of all
          canisters on the IC. Coverage percentage is shown in the header.
        </li>
        <li>
          <strong>Top-up detection:</strong> Very small top-ups (&lt; 500B cycles) aren't detected
          and may slightly skew burn rates.
        </li>
        <li>
          <strong>Cycle transfers:</strong> We identify known cycle-transferring canisters, but
          new ones may not be immediately flagged.
        </li>
      </ul>
    </section>
  </div>

  <footer>
    <a href="/">Back to Leaderboard</a>
    <span class="meta-sep">·</span>
    An <a href="https://alexandriadao.com/" target="_blank" rel="noopener">Alexandria</a> Project
  </footer>
</div>

<style>
  .back-link {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    padding: 4px;
    border-radius: 4px;
    transition: all 0.15s ease;
  }

  .back-link:hover {
    color: var(--accent);
    background: var(--bg-tertiary);
  }

  .page-title {
    font-size: 24px;
    font-weight: 600;
    margin-top: 16px;
    color: var(--text);
  }

  .content {
    max-width: 800px;
  }

  .methodology-section {
    margin-bottom: 40px;
  }

  .methodology-section h2 {
    font-size: 18px;
    font-weight: 600;
    color: var(--accent);
    margin-bottom: 12px;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border);
  }

  .methodology-section h3 {
    font-size: 15px;
    font-weight: 600;
    color: var(--text);
    margin-top: 20px;
    margin-bottom: 8px;
  }

  .methodology-section p {
    line-height: 1.7;
    margin-bottom: 12px;
    color: var(--text);
  }

  .methodology-section ul {
    margin: 12px 0;
    padding-left: 24px;
  }

  .methodology-section li {
    line-height: 1.7;
    margin-bottom: 8px;
    color: var(--text);
  }

  .info-box {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-left: 3px solid var(--accent);
    border-radius: 6px;
    padding: 12px 16px;
    margin: 16px 0;
    font-size: 13px;
    color: var(--text-muted);
  }

  .info-box strong {
    color: var(--text);
  }

  .formula {
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 16px 20px;
    font-family: "SF Mono", Monaco, Consolas, monospace;
    font-size: 14px;
    text-align: center;
    margin: 16px 0;
    color: var(--accent);
  }

  .formula-note {
    font-size: 13px;
    color: var(--text-muted);
    font-style: italic;
  }

  .info-table {
    width: 100%;
    border-collapse: collapse;
    margin: 16px 0;
    font-size: 13px;
  }

  .info-table th,
  .info-table td {
    text-align: left;
    padding: 10px 12px;
    border-bottom: 1px solid var(--border);
  }

  .info-table th {
    background: var(--bg-secondary);
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    font-size: 11px;
  }

  .info-table td {
    color: var(--text);
  }

  .badge-example {
    display: inline-block;
    background: var(--orange);
    color: var(--bg);
    font-weight: 600;
    font-size: 11px;
    padding: 2px 6px;
    border-radius: 4px;
    margin: 0 2px;
  }

  .runway-critical {
    color: var(--red);
    font-weight: 600;
  }

  .runway-warning {
    color: var(--orange);
  }

  .runway-ok {
    color: var(--text);
  }

  .runway-good {
    color: var(--green);
  }

  .runway-infinite {
    color: var(--text-muted);
  }

  a {
    color: var(--accent);
    text-decoration: none;
  }

  a:hover {
    text-decoration: underline;
  }

  footer {
    display: flex;
    align-items: center;
    gap: 8px;
    justify-content: center;
  }
</style>
