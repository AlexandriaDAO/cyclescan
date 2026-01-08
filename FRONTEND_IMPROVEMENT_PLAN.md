# CycleScan Frontend Improvement Plan

## Executive Summary

After deep analysis, I've identified improvements across 7 dimensions: **Performance**, **User Experience**, **Visual Design**, **Features**, **Information Architecture**, **Code Quality**, and **Data Accuracy**. This plan prioritizes high-impact changes.

---

## 1. PERFORMANCE (Critical)

### Problem: 9.5MB JSON Load + Client-Side Calculations

Every page load:
1. Downloads ~9.5MB `snapshots.json` (168 snapshots × 2900 canisters)
2. Calculates regression for every canister (CPU-intensive)
3. No caching beyond in-memory

### Solutions

#### 1.1 Pre-Calculate Burn Rates (High Impact)
Move regression calculations to the GitHub Actions collection script.

**Current Flow:**
```
GitHub Actions → snapshots.json → Browser downloads → Browser calculates rates
```

**Proposed Flow:**
```
GitHub Actions → Calculate rates → rates.json (small) → Browser displays
                              → snapshots.json (archive, optional download)
```

**New `rates.json` structure (~200KB vs 9.5MB):**
```json
{
  "generated_at": 1704729600000,
  "projects": [
    {
      "name": "OpenChat",
      "canister_count": 45,
      "total_balance": "523000000000000",
      "rates": {
        "recent": { "rate": "450000000000", "confidence": 0.92, "unreliable": false },
        "short_term": { "rate": "420000000000", "confidence": 0.95, "unreliable": false },
        "long_term": { "rate": "410000000000", "confidence": 0.97, "unreliable": false }
      },
      "runway_days": 1245
    }
  ],
  "canisters": { /* indexed by canister_id */ }
}
```

**Benefits:**
- 98% reduction in download size
- Zero client-side calculation
- Faster time-to-interactive

#### 1.2 Lazy Load Chart Library
```javascript
// Current: Always loaded
import { createChart } from "lightweight-charts";

// Proposed: Load on modal open
const { createChart } = await import("lightweight-charts");
```

#### 1.3 Virtual Scrolling for Large Tables
Replace pagination with virtual scrolling for smoother experience with 200+ projects.

---

## 2. USER EXPERIENCE

### Problem: State Not Preserved

- Refresh loses: sort order, expanded projects, search, page
- Can't share links to specific views
- No mobile optimization

### Solutions

#### 2.1 URL-Based State (High Impact)
```
https://cyclescan.io/?sort=burn&dir=desc&search=open&expand=openchat,kinic
https://cyclescan.io/canister/ryjl3-tyaaa-aaaaa-aaaba-cai
```

**Implementation:**
```javascript
import { page } from '$app/stores';
import { goto } from '$app/navigation';

// Sync state with URL
$: {
  const params = new URLSearchParams();
  if (sortColumn !== 'short_term_rate') params.set('sort', sortColumn);
  if (sortDirection !== 'desc') params.set('dir', sortDirection);
  if (searchQuery) params.set('q', searchQuery);
  goto(`?${params}`, { replaceState: true, noScroll: true });
}
```

#### 2.2 Search Improvements
- Search canister IDs, not just project names
- Fuzzy matching for typo tolerance
- Recent searches memory

#### 2.3 Mobile-First Responsive Design
```
Desktop: Full table with all columns
Tablet: Hide recent rate, show on tap
Mobile: Card view with key metrics only
```

#### 2.4 Keyboard Navigation
- `j/k` - Move up/down in table
- `Enter` - Expand project / Open modal
- `Escape` - Close modal / Collapse
- `/` - Focus search

---

## 3. VISUAL DESIGN

### Problem: Dense Data, Weak Hierarchy

Everything looks equally important. Hard to scan quickly.

### Solutions

#### 3.1 Sparklines (High Impact)
Add 7-day trend sparkline in each row:

```
┌─────────────────────────────────────────────────────────────┐
│ #  Project      Cans  Balance   Trend    Burn/day   Runway │
├─────────────────────────────────────────────────────────────┤
│ 1  OpenChat     45    523T     ▂▃▅▇▆▅▄   420B/day   3.4y   │
│ 2  Kinic        12    89T      ▁▁▂▃▅▇▇   180B/day   1.4y   │
│ 3  BOOM DAO     8     12T      ▇▆▅▄▃▂▁   25B/day    1.3y   │ ← declining
└─────────────────────────────────────────────────────────────┘
```

**Implementation:** Use `<canvas>` or SVG for lightweight sparklines (no library needed).

#### 3.2 Critical Runway Highlighting
```css
/* Pulsing red for <7 days runway */
.runway-critical {
  animation: pulse-critical 2s infinite;
}

@keyframes pulse-critical {
  0%, 100% { background: rgba(248, 81, 73, 0.1); }
  50% { background: rgba(248, 81, 73, 0.25); }
}
```

#### 3.3 Confidence Visualization
Replace `~` badge with visual confidence bar:

```
High confidence:   ████████░░ 85%
Low confidence:    ███░░░░░░░ 32%  ⚠️
```

#### 3.4 Chart Enhancements
- Mark top-up events on chart (vertical lines)
- Add moving average overlay
- Show comparison to network average
- Zoom/pan with touch support

---

## 4. NEW FEATURES

### 4.1 Historical Trends Dashboard (High Impact)
New `/trends` page showing:
- How has this project's burn changed over time?
- Network-wide burn trends
- Top gainers/losers this week

```
OpenChat Burn Rate History
─────────────────────────────
            ▂▃▄▅▆▇▆▅▄▃▂▁
Jan 1      Jan 7      Jan 14

Current: 420B/day
7d avg:   385B/day  (+9.1%)
30d avg:  410B/day  (+2.4%)
```

### 4.2 Alerts & Notifications
- Low runway warnings (configurable threshold)
- Burn rate spike alerts
- Top-up notifications

**Implementation:** Browser notifications + optional email via simple form

### 4.3 Comparison Mode
Select 2-4 projects to compare side-by-side:
- Burn rate trends
- Runway projections
- Balance history

### 4.4 Export Functionality
- CSV export of current view
- JSON export for developers
- Shareable report links

### 4.5 Canister Health Score
Combine metrics into single score:
```
Health Score = weighted(
  runway_score,      // 40%
  confidence_score,  // 30%
  trend_score,       // 20%
  balance_score      // 10%
)
```

Display as A/B/C/D/F or 0-100 score.

---

## 5. INFORMATION ARCHITECTURE

### Problem: Single View, No Filtering

Only one way to see data (project leaderboard).

### Solutions

#### 5.1 Multiple Views
```
┌─────────────────────────────────────────────┐
│ [Projects] [Canisters] [Analytics] [Alerts] │
└─────────────────────────────────────────────┘
```

- **Projects:** Current view (default)
- **Canisters:** Flat list of all canisters, sortable
- **Analytics:** Network-wide stats, trends, comparisons
- **Alerts:** Configure and view alerts

#### 5.2 Advanced Filters
```
┌─────────────────────────────────────────────┐
│ Filters:                                    │
│ ┌─────────────┐ ┌─────────────┐            │
│ │ Runway ▼    │ │ Burn Rate ▼ │            │
│ └─────────────┘ └─────────────┘            │
│ ☐ < 30 days    ☐ > 1T/day                  │
│ ☐ 30-90 days   ☐ 100B-1T/day               │
│ ☐ > 90 days    ☐ < 100B/day                │
│                                             │
│ ┌─────────────┐ ┌─────────────┐            │
│ │ Type ▼      │ │ Confidence ▼│            │
│ └─────────────┘ └─────────────┘            │
│ ☐ SNS         ☐ High (>0.8)                │
│ ☐ Non-SNS     ☐ Medium                     │
│               ☐ Low (<0.5)                  │
└─────────────────────────────────────────────┘
```

#### 5.3 Inline Help
Replace separate About page with contextual tooltips:
- Hover on "Burn Rate" header → tooltip explains methodology
- Hover on "~" → explains unreliable data
- First-time visitor tutorial overlay

---

## 6. CODE QUALITY

### Current Issues
- 820-line monolithic page component
- Duplicated formatting functions
- Legacy fields cluttering interfaces
- No tests for critical calculations

### Solutions

#### 6.1 Component Extraction
```
src/lib/components/
├── ProjectTable.svelte        # Main table
├── ProjectRow.svelte          # Single project row
├── CanisterSubRow.svelte      # Expanded canister row
├── Sparkline.svelte           # Trend visualization
├── RateCell.svelte            # Burn rate display
├── RunwayBadge.svelte         # Runway indicator
├── FilterPanel.svelte         # Advanced filters
├── SearchBar.svelte           # Enhanced search
└── CanisterDetailModal.svelte # (existing)
```

#### 6.2 Shared Utilities
```typescript
// src/lib/format.ts
export function formatCycles(value: bigint): string;
export function formatRate(ratePerHour: bigint): FormattedRate;
export function formatRunway(days: number | null): FormattedRunway;
export function formatTimeDelta(hours: number): string;
```

#### 6.3 Remove Legacy Fields
Clean migration path:
1. Stop using legacy fields in UI (done)
2. Remove from interfaces
3. Remove from data loading

#### 6.4 Add Tests
```typescript
// src/lib/regression.test.ts
describe('linearRegression', () => {
  it('calculates correct slope for linear data', () => {});
  it('returns null for insufficient data', () => {});
  it('handles top-ups correctly', () => {});
});
```

---

## 7. DATA ACCURACY

### Issues
- "Recent" rate often has only 2-3 data points
- Project aggregation uses simple sum (R² doesn't sum)
- No data freshness indicator

### Solutions

#### 7.1 Data Quality Indicators
Show users when data might be less reliable:
```
Last updated: 45 minutes ago
Data quality: ████████░░ Good (168 snapshots)
```

#### 7.2 Smarter Recent Rate
For 2-hour window with <3 points, show simple difference instead of regression:
```javascript
if (points.length < 3) {
  // Simple point-to-point calculation
  return calculateSimpleBurn(points);
} else {
  // Full regression
  return linearRegression(points);
}
```

#### 7.3 Better Project Aggregation
Instead of summing rates, calculate project-level regression directly from canister balances:
```javascript
// Sum all canister balances at each timestamp, then regress
const projectPoints = timestamps.map(t => ({
  t,
  v: canisters.reduce((sum, c) => sum + c.balanceAt(t), 0)
}));
return linearRegression(projectPoints);
```

---

## Implementation Priority

### Phase 1: Quick Wins (1-2 days)
1. ✅ URL state persistence
2. ✅ Data freshness indicator
3. ✅ Sparklines in table rows
4. ✅ Component extraction (start)

### Phase 2: Performance (3-5 days)
1. Pre-calculate rates server-side
2. Create `rates.json` endpoint
3. Lazy load chart library
4. Mobile responsive design

### Phase 3: Features (1-2 weeks)
1. Advanced filters
2. Comparison mode
3. Export functionality
4. Historical trends page

### Phase 4: Polish (1 week)
1. Alerts system
2. Health score
3. Keyboard navigation
4. Tutorial overlay

---

## Technical Notes

### Pre-calculation Script Changes
```javascript
// scripts/collect_snapshots.mjs additions

async function calculateAllRates(snapshots) {
  const rates = {};
  for (const canisterId of Object.keys(snapshots[0].balances)) {
    rates[canisterId] = calculateBurnRateWithTopUps(snapshots, canisterId);
  }
  return rates;
}

// After collecting snapshots
const rates = calculateAllRates(snapshots);
await fs.writeFile('data/live/rates.json', JSON.stringify(rates));
```

### Sparkline Component
```svelte
<!-- Sparkline.svelte - ~30 lines, no dependencies -->
<script>
  export let data = []; // Array of values
  export let width = 60;
  export let height = 20;

  $: points = data.map((v, i) => {
    const x = (i / (data.length - 1)) * width;
    const y = height - (v / Math.max(...data)) * height;
    return `${x},${y}`;
  }).join(' ');
</script>

<svg {width} {height}>
  <polyline
    fill="none"
    stroke="currentColor"
    stroke-width="1.5"
    points={points}
  />
</svg>
```

---

## Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Initial load size | ~9.5MB | <500KB |
| Time to interactive | ~3s | <1s |
| Mobile usability score | ~60 | >90 |
| Page views/session | ~2 | >5 |

---

## Conclusion

The biggest wins come from:
1. **Pre-calculating rates** → 98% smaller downloads, instant display
2. **Sparklines** → Immediate visual understanding of trends
3. **URL state** → Shareable, bookmarkable views
4. **Mobile design** → Reach more users

Start with Phase 1 quick wins to demonstrate value, then tackle performance in Phase 2.
