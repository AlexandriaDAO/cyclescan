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
      <img src="/cyclescan_canister.png" alt="CycleScan" class="header-logo" />
      <span class="brand-name">CycleScan</span>
    </div>
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
        CycleScan uses an <strong>interval-based averaging</strong> approach to calculate burn rates.
        We analyze the change in balance between each consecutive snapshot:
      </p>
      <ul>
        <li><strong>Burn intervals:</strong> When balance decreases between snapshots, this represents actual cycle consumption</li>
        <li><strong>Top-up intervals:</strong> When balance increases, this indicates a cycle top-up occurred</li>
      </ul>

      <h3>The Algorithm</h3>
      <p>
        For each canister within a time window, we:
      </p>
      <ol>
        <li>Calculate the burn rate from all intervals where the balance decreased</li>
        <li>Compute the average burn rate per millisecond from these "burn intervals"</li>
        <li>For top-up intervals, we <strong>infer</strong> what the burn would have been based on this average</li>
        <li>Sum actual burns + inferred burns to get the total burn over the full time period</li>
      </ol>
      <div class="formula">
        Burn Rate = (Actual Burns + Inferred Burns) ÷ Total Duration
      </div>
      <p class="formula-note">
        This approach gives us accurate burn rates even when top-ups occur, because we estimate
        the burn that would have happened during those periods.
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
        When a canister receives new cycles (a "top-up"), its balance jumps upward. CycleScan
        detects <strong>any balance increase</strong> as a top-up and handles it automatically:
      </p>
      <ul>
        <li>
          <strong>Detection:</strong> Any interval where the ending balance is higher than the
          starting balance is marked as a top-up interval.
        </li>
        <li>
          <strong>Inference:</strong> During top-up intervals, we estimate the burn that would
          have occurred based on the average burn rate from non-top-up intervals.
        </li>
        <li>
          <strong>Calculation:</strong> The final burn rate includes both actual measured burns
          and inferred burns, giving you an accurate picture of consumption.
        </li>
      </ul>
      <div class="info-box">
        <strong>Example:</strong> If a canister burns 100B cycles/hour on average, and receives
        a 1T cycle top-up during a 1-hour interval, we infer that ~100B cycles were also burned
        during that hour. The displayed burn rate accounts for this.
      </div>
    </section>

    <section class="methodology-section">
      <h2>Chart Visualization</h2>
      <p>
        When you click on a canister, the detail chart shows burn rates per interval with
        color-coded bars:
      </p>
      <table class="info-table">
        <tr>
          <th>Color</th>
          <th>Meaning</th>
        </tr>
        <tr>
          <td><span class="color-dot actual"></span> Green</td>
          <td><strong>Actual burn</strong> - Measured cycle consumption from intervals where balance decreased</td>
        </tr>
        <tr>
          <td><span class="color-dot inferred"></span> Orange</td>
          <td><strong>Inferred burn</strong> - Estimated consumption during top-up intervals, based on average burn rate</td>
        </tr>
        <tr>
          <td><span class="color-dot topup"></span> Red (below zero)</td>
          <td><strong>Top-up amount</strong> - The cycles added during that interval (shown as negative/below the axis)</td>
        </tr>
      </table>
      <p>
        The orange dashed line shows the average burn rate across all actual burn intervals.
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
          <td class="runway-good">&gt; 1y</td>
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
      <h2>Get Your Project Listed</h2>
      <p>
        Missing from CycleScan? Here's how to get your project added.
      </p>
      <h3>Eligibility Requirements</h3>
      <p>
        Your canister must have a controller that publicly exposes cycle balance information.
        This means one of the following must be a controller of your canister:
      </p>
      <ul>
        <li>
          <strong>Blackhole canister</strong> (<code>e3mmv-5qaaa-aaaah-aadma-cai</code>) -
          A widely-used canister that allows anyone to query <code>canister_status</code>
        </li>
        <li>
          <strong>NNS Root canister</strong> (<code>r7inp-6aaaa-aaaaa-aaabq-cai</code>) -
          The NNS root also exposes <code>canister_status</code> publicly
        </li>
        <li>
          <strong>SNS Root canister</strong> - If your project is an SNS, cycle data is
          automatically available via <code>get_sns_canisters_summary()</code>
        </li>
      </ul>

      <h3>About the Blackhole Canister</h3>
      <p>
        The blackhole canister is the recommended option for most projects. It's a simple, trustworthy
        canister created by <a href="https://github.com/ninegua/ic-blackhole" target="_blank" rel="noopener">ninegua</a>
        with these properties:
      </p>
      <ul>
        <li>
          <strong>Open source</strong> - The entire code is ~25 lines of Motoko, just a thin wrapper
          around the IC management canister
        </li>
        <li>
          <strong>Immutable</strong> - Its only controller is itself, so it can never be modified
        </li>
        <li>
          <strong>Safe</strong> - It only exposes <code>canister_status</code> for reading. It cannot
          upgrade, stop, delete, or modify your canister in any way
        </li>
        <li>
          <strong>Verified</strong> - Module hash: <code>0x210cf9...9d7de0</code>
        </li>
      </ul>
      <p>
        The only "downside" is that anyone can query your canister's cycle balance, memory usage, and
        module hash. For most projects, this transparency is a feature, not a bug.
      </p>

      <div class="info-box">
        <strong>Why is this required?</strong> Without one of these controllers, there's no way to
        publicly query a canister's cycle balance. The IC management canister's <code>canister_status</code>
        method is restricted to controllers, so a public proxy like the blackhole is needed.
      </div>
      <h3>How to Get Added</h3>
      <p>
        Once your canister meets the eligibility requirements, reach out to us on X (Twitter):
      </p>
      <p style="text-align: center; margin: 20px 0;">
        <a href="https://x.com/alexandria_lbry" target="_blank" rel="noopener" class="contact-link">
          @alexandria_lbry
        </a>
      </p>
      <p>
        Include your canister ID(s) and project name, and we'll add you to the leaderboard.
      </p>
      <div class="info-box">
        <strong>Background:</strong> CycleScan was initiated by querying every single canister on
        the entire ICP network and checking for eligibility. However, we don't continuously scan
        for newly eligible canisters, so if you've recently added the blackhole as a controller
        or launched a new project, you'll need to let us know.
      </div>
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
          <strong>Inferred burn accuracy:</strong> When top-ups occur, the inferred burn is based
          on the average from other intervals. If burn rates vary significantly, the inference
          may be less accurate.
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

  .color-dot {
    display: inline-block;
    width: 12px;
    height: 12px;
    border-radius: 3px;
    margin-right: 6px;
    vertical-align: middle;
  }

  .color-dot.actual {
    background: #00d395;
  }

  .color-dot.inferred {
    background: #f97316;
  }

  .color-dot.topup {
    background: #f85149;
  }

  .methodology-section ol {
    margin: 12px 0;
    padding-left: 24px;
  }

  .methodology-section ol li {
    line-height: 1.7;
    margin-bottom: 8px;
    color: var(--text);
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

  .contact-link {
    display: inline-block;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    padding: 12px 24px;
    border-radius: 8px;
    font-weight: 600;
    font-size: 16px;
    transition: all 0.15s ease;
  }

  .contact-link:hover {
    background: var(--bg-tertiary);
    border-color: var(--accent);
    text-decoration: none;
  }

  code {
    background: var(--bg-tertiary);
    padding: 2px 6px;
    border-radius: 4px;
    font-family: "SF Mono", Monaco, Consolas, monospace;
    font-size: 12px;
    color: var(--text-muted);
  }

  footer {
    display: flex;
    align-items: center;
    gap: 8px;
    justify-content: center;
  }
</style>
