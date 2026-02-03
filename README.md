# FrankenTUI (ftui)

```
███████╗██████╗  █████╗ ███╗   ██╗██╗  ██╗███████╗███╗   ██╗████████╗██╗   ██╗██╗
██╔════╝██╔══██╗██╔══██╗████╗  ██║██║ ██╔╝██╔════╝████╗  ██║╚══██╔══╝██║   ██║██║
█████╗  ██████╔╝███████║██╔██╗ ██║█████╔╝ █████╗  ██╔██╗ ██║   ██║   ██║   ██║██║
██╔══╝  ██╔══██╗██╔══██║██║╚██╗██║██╔═██╗ ██╔══╝  ██║╚██╗██║   ██║   ██║   ██║██║
██║     ██║  ██║██║  ██║██║ ╚████║██║  ██╗███████╗██║ ╚████║   ██║   ╚██████╔╝██║
╚═╝     ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═══╝   ╚═╝    ╚═════╝ ╚═╝
```

<div align="center">
  <img src="frankentui_illustration.webp" alt="FrankenTUI - Minimal, high-performance terminal UI kernel">
</div>

Minimal, high‑performance terminal UI kernel focused on correctness, determinism, and clean architecture.

![status](https://img.shields.io/badge/status-WIP-yellow)
![rust](https://img.shields.io/badge/rust-nightly-blue)
![license](https://img.shields.io/badge/license-unspecified-lightgrey)

## Quick Run (from source)

```bash
# Download source with curl (no installer yet)
curl -fsSL https://codeload.github.com/Dicklesworthstone/frankentui/tar.gz/main | tar -xz
cd frankentui-main

# Run the reference harness app
cargo run -p ftui-harness
```

**Or clone with git:**

```bash
git clone https://github.com/Dicklesworthstone/frankentui.git
cd frankentui
cargo run -p ftui-harness
```

---

## TL;DR

**The Problem:** Most TUI stacks make it easy to draw widgets, but hard to build *correct*, *flicker‑free*, *inline* UIs with strict terminal cleanup and deterministic rendering.

**The Solution:** FrankenTUI is a kernel‑level TUI foundation with a disciplined runtime, diff‑based renderer, and inline‑mode support that preserves scrollback while keeping UI chrome stable.

### Why Use FrankenTUI?

| Feature | What It Does | Example |
|---------|--------------|---------|
| **Inline mode** | Stable UI at top/bottom while logs scroll above | `ScreenMode::Inline { ui_height: 10 }` in the runtime |
| **Deterministic rendering** | Buffer → Diff → Presenter → ANSI, no hidden I/O | `BufferDiff::compute(&prev, &next)` |
| **One‑writer rule** | Serializes output for correctness | `TerminalWriter` owns all stdout writes |
| **RAII cleanup** | Terminal state restored even on panic | `TerminalSession` in `ftui-core` |
| **Composable crates** | Layout, text, style, runtime, widgets | Add only what you need |

---

## Quick Example

```bash
# Reference app (inline mode, log streaming)
FTUI_HARNESS_SCREEN_MODE=inline FTUI_HARNESS_UI_HEIGHT=12 cargo run -p ftui-harness

# Try other harness views
FTUI_HARNESS_VIEW=layout-grid cargo run -p ftui-harness
FTUI_HARNESS_VIEW=widget-table cargo run -p ftui-harness
```

---

## Design Philosophy

1. **Correctness over cleverness** — predictable terminal state is non‑negotiable.
2. **Deterministic output** — buffer diffs and explicit presentation over ad‑hoc writes.
3. **Inline first** — preserve scrollback while keeping chrome stable.
4. **Layered architecture** — core → render → runtime → widgets, no cyclic dependencies.
5. **Zero‑surprise teardown** — RAII cleanup, even when apps crash.

---

## Workspace Overview

| Crate | Purpose | Status |
|------|---------|--------|
| `ftui` | Public facade + prelude | Implemented |
| `ftui-core` | Terminal lifecycle, events, capabilities | Implemented |
| `ftui-render` | Buffer, diff, ANSI presenter | Implemented |
| `ftui-style` | Style + theme system | Implemented |
| `ftui-text` | Spans, segments, rope editor | Implemented |
| `ftui-layout` | Flex + Grid solvers | Implemented |
| `ftui-runtime` | Elm/Bubbletea runtime | Implemented |
| `ftui-widgets` | Core widget library | Implemented |
| `ftui-extras` | Feature‑gated add‑ons | Implemented |
| `ftui-harness` | Reference app + snapshots | Implemented |
| `ftui-pty` | PTY test utilities | Implemented |
| `ftui-simd` | Optional safe optimizations | Reserved |

---

## How FrankenTUI Compares

| Feature | FrankenTUI | Ratatui | tui-rs (legacy) | Raw crossterm |
|---------|------------|---------|-----------------|---------------|
| Inline mode w/ scrollback | ✅ First‑class | ⚠️ App‑specific | ⚠️ App‑specific | ❌ Manual |
| Deterministic buffer diff | ✅ Kernel‑level | ✅ | ✅ | ❌ |
| One‑writer rule | ✅ Enforced | ⚠️ App‑specific | ⚠️ App‑specific | ❌ |
| RAII teardown | ✅ TerminalSession | ⚠️ App‑specific | ⚠️ App‑specific | ❌ |
| Snapshot/time‑travel harness | ✅ Built‑in | ❌ | ❌ | ❌ |

**When to use FrankenTUI:**
- You want inline + scrollback without flicker.
- You care about deterministic rendering and teardown guarantees.
- You prefer a kernel you can build your own UI framework on top of.

**When FrankenTUI might not be ideal:**
- You need a huge widget ecosystem today (FrankenTUI is still early stage).
- You want a fully opinionated application framework rather than a kernel.

---

## Installation

### Quick Install (Source Tarball)

```bash
curl -fsSL https://codeload.github.com/Dicklesworthstone/frankentui/tar.gz/main | tar -xz
cd frankentui-main
cargo build --release
```

### Git Clone

```bash
git clone https://github.com/Dicklesworthstone/frankentui.git
cd frankentui
cargo build --release
```

### Use as a Workspace Dependency

```toml
# Cargo.toml
[dependencies]
ftui = { path = "../frankentui/crates/ftui" }
```

---

## Quick Start

1. **Install Rust nightly** (required by `rust-toolchain.toml`).
2. **Clone the repo** and build:
   ```bash
   git clone https://github.com/Dicklesworthstone/frankentui.git
   cd frankentui
   cargo build
   ```
3. **Run the reference harness:**
   ```bash
   cargo run -p ftui-harness
   ```

---

## Telemetry (Optional)

Telemetry is opt‑in. Enable the `telemetry` feature on `ftui-runtime` and set
OTEL env vars (for example, `OTEL_EXPORTER_OTLP_ENDPOINT`) to export spans.

When the feature is **off**, telemetry code and dependencies are excluded.
When the feature is **on** but env vars are unset, overhead is a single
startup check.

See `docs/telemetry.md` for integration patterns and trace‑parent attachment.

---

## Commands

### Run the Harness

```bash
cargo run -p ftui-harness
```

### Run Harness Examples

```bash
cargo run -p ftui-harness --example minimal
cargo run -p ftui-harness --example streaming
```

### Tests

```bash
cargo test
BLESS=1 cargo test -p ftui-harness  # update snapshot baselines
```

### Format + Lint

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
```

### E2E Scripts

```bash
./scripts/e2e_test.sh
./scripts/widget_api_e2e.sh
```

---

## Configuration

FrankenTUI is configuration‑light. The harness is configured via environment variables:

```bash
# .env (example)
FTUI_HARNESS_SCREEN_MODE=inline   # inline | alt
FTUI_HARNESS_UI_HEIGHT=12         # rows reserved for UI
FTUI_HARNESS_VIEW=layout-grid     # view selector
FTUI_HARNESS_ENABLE_MOUSE=true
FTUI_HARNESS_ENABLE_FOCUS=true
FTUI_HARNESS_LOG_LINES=25
FTUI_HARNESS_LOG_MARKUP=true
FTUI_HARNESS_LOG_FILE=/path/to/log.txt
FTUI_HARNESS_EXIT_AFTER_MS=0      # 0 disables auto-exit
```

Terminal capability detection uses standard environment variables (`TERM`, `COLORTERM`, `NO_COLOR`, `TMUX`, `ZELLIJ`, `KITTY_WINDOW_ID`).

---

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                          Input Layer                              │
│   TerminalSession (crossterm) → Event (ftui-core)                 │
└──────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌──────────────────────────────────────────────────────────────────┐
│                          Runtime Loop                              │
│   Program/Model (ftui-runtime) → Cmd → Subscriptions              │
└──────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌──────────────────────────────────────────────────────────────────┐
│                         Render Kernel                              │
│   Frame → Buffer → BufferDiff → Presenter → ANSI                  │
└──────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌──────────────────────────────────────────────────────────────────┐
│                          Output Layer                              │
│   TerminalWriter (inline or alt-screen)                           │
└──────────────────────────────────────────────────────────────────┘
```

---

## "Alien Artifact" Quality Algorithms

FrankenTUI employs mathematically rigorous algorithms that go far beyond typical TUI implementations—what we call "alien artifact" quality engineering.

### Bayesian Fuzzy Scoring (Command Palette)

The command palette uses a **Bayesian evidence ledger** for match scoring, not simple string distance:

```
Score = P(relevant | evidence) computed via posterior odds:

P(relevant | evidence) / P(not_relevant | evidence)
    = [P(relevant) / P(not_relevant)] × Π_i BF_i

where BF_i = Bayes Factor for evidence type i
          = P(evidence_i | relevant) / P(evidence_i | not_relevant)
```

**Prior odds by match type:**
| Match Type | Prior Odds | P(relevant) | Intuition |
|------------|------------|-------------|-----------|
| Exact | 99:1 | 99% | Almost always what user wants |
| Prefix | 9:1 | 90% | Very likely relevant |
| Word-start | 4:1 | 80% | Probably relevant |
| Substring | 2:1 | 67% | Possibly relevant |
| Fuzzy | 1:3 | 25% | Needs additional evidence |

**Evidence factors that update posterior:**
- **Word boundary bonus** (BF ≈ 2.0): Match at start of word
- **Position bonus** (BF ∝ 1/position): Earlier matches stronger
- **Gap penalty** (BF < 1.0): Gaps between matched chars reduce confidence
- **Tag match bonus** (BF ≈ 3.0): Query matches command tags
- **Length factor** (BF ∝ 1/length): Shorter, more specific titles preferred

**Result:** Every search result includes an explainable evidence ledger showing exactly why it ranked where it did.

### Bayesian Diff Strategy Selection

The renderer adaptively chooses between diff strategies using a **Beta posterior over change rates**:

```
Change-rate model:
    p ~ Beta(α, β)

Prior: α₀ = 1, β₀ = 19  →  E[p] = 5% (expect sparse changes)

Per-frame update:
    α ← α × decay + N_changed
    β ← β × decay + (N_scanned - N_changed)

where decay = 0.95 (exponential forgetting for non-stationary workloads)
```

**Strategy cost model:**
```
Cost = c_scan × cells_scanned + c_emit × cells_emitted

Full Diff:     Cost = c_row × H + c_scan × D × W + c_emit × p × N
Dirty-Row:     Cost = c_scan × D × W + c_emit × p × N
Full Redraw:   Cost = c_emit × N

Decision: argmin { E[Cost_full], E[Cost_dirty], E[Cost_redraw] }
```

**Conservative mode:** Uses 95th percentile of p (not mean) when posterior variance is high—the system knows when it's uncertain.

### BOCPD: Online Change-Point Detection

Resize coalescing uses **Bayesian Online Change-Point Detection** to detect regime transitions:

```
Observation model (inter-arrival times):
    Steady: x_t ~ Exponential(λ_steady)  where μ_steady ≈ 200ms
    Burst:  x_t ~ Exponential(λ_burst)   where μ_burst ≈ 20ms

Run-length posterior (recursive update):
    P(r_t = 0 | x_1:t) ∝ Σᵣ P(r_{t-1} = r) × H(r) × P(x_t | r)
    P(r_t = r+1 | x_1:t) ∝ P(r_{t-1} = r) × (1 - H(r)) × P(x_t | r)

Hazard function (geometric prior):
    H(r) = 1/λ_hazard  where λ_hazard = 50

Complexity: O(K) per update with K=100 run-length truncation
```

**Regime posterior:**
```
P(burst | observations) = Σᵣ P(burst | r, x_1:t) × P(r | x_1:t)

Decision thresholds:
    p_burst > 0.7  →  Burst regime (aggressive coalescing)
    p_burst < 0.3  →  Steady regime (responsive)
    otherwise      →  Transitional (interpolate delay)
```

### Value-of-Information (VOI) Sampling

Expensive operations (height remeasurement, full diff) use **VOI analysis** to decide when to sample:

```
Beta posterior over violation probability:
    p ~ Beta(α, β)

VOI computation:
    variance_before = αβ / ((α+β)² × (α+β+1))
    variance_after  = (α+1)β / ((α+β+2)² × (α+β+3))  [if success]
    VOI = variance_before - E[variance_after]

Decision:
    sample iff (max_interval exceeded) OR (VOI × value_scale ≥ sample_cost)
```

**Tuned defaults for TUI:**
- `prior_alpha=1.0, prior_beta=9.0` (expect 10% violation rate)
- `max_interval_ms=1000` (latency bound)
- `min_interval_ms=100` (prevent over-sampling)
- `sample_cost=0.08` (moderately expensive)

### E-Process: Anytime-Valid Testing

All statistical thresholds use **e-processes** (wealth-based sequential tests):

```
Wealth process:
    W_t = W_{t-1} × (1 + λ_t × (X_t - μ₀))

where λ_t is the betting fraction from GRAPA (General Random Adaptive Proportion Algorithm)

Key guarantee:
    P(∃t: W_t ≥ 1/α) ≤ α   under null hypothesis

This holds at ANY stopping time—no peeking penalty!
```

**Applications in FrankenTUI:**
- Budget degradation decisions
- Flake detection in tests
- Allocation budget alerts
- Conformal prediction thresholds

### Conformal Alerting

Budget and performance alerts use **distribution-free conformal prediction**:

```
Nonconformity score:
    R_t = |observed_t - predicted_t|

Threshold (finite-sample guarantee):
    q = quantile_{(1-α)(n+1)/n}(R_1, ..., R_n)

Coverage guarantee:
    P(R_{n+1} ≤ q) ≥ 1 - α   for any distribution!

E-process layer (anytime-valid):
    e_t = exp(λ × (z_t - μ₀) - λ²σ²/2)
```

**Why conformal?** No distributional assumptions required—works for any data pattern.

### CUSUM Control Charts

Allocation budget tracking uses **CUSUM** (Cumulative Sum) for fast change detection:

```
One-sided CUSUM:
    S_t = max(0, S_{t-1} + (X_t - μ₀) - k)

Alert when:
    S_t > h (threshold)

Parameters:
    k = allowance (typically σ/2)
    h = threshold (controls sensitivity vs false alarms)

Dual detection:
    Alert iff (CUSUM detects AND e-process confirms)
           OR (e-process alone exceeds 1/α)
```

**Why dual?** CUSUM is fast but can false-alarm; e-process is slower but anytime-valid. Intersection gives speed with guarantees.

### Jain's Fairness Index (Input Guard)

Input fairness monitoring uses **Jain's Fairness Index**:

```
F(x₁, ..., xₙ) = (Σxᵢ)² / (n × Σxᵢ²)

Properties:
    F = 1.0  →  Perfect fairness (all equal)
    F = 1/n  →  Complete unfairness (one dominates)

Intervention:
    if input_latency > threshold OR F < 0.8:
        force_coalescer_yield()
```

**Why Jain's?** Scale-independent, bounded [1/n, 1], interpretable.

---

## Troubleshooting

### "terminal is corrupted after crash"

FrankenTUI uses RAII cleanup via `TerminalSession`. If you see a broken terminal, make sure you are not force‑killing the process.

```bash
# Reset terminal state
reset
```

### “error: the option `-Z` is only accepted on the nightly compiler”

FrankenTUI requires nightly. Install and use nightly or let `rust-toolchain.toml` select it.

```bash
rustup toolchain install nightly
```

### “raw mode not restored”

Ensure your app exits normally (or panics) and does not call `process::exit()` before `TerminalSession` drops.

### “no mouse events”

Mouse must be enabled in the session and supported by your terminal.

```bash
FTUI_HARNESS_ENABLE_MOUSE=true cargo run -p ftui-harness
```

### “output flickers”

Inline mode uses synchronized output where supported. If you’re in a very old terminal or multiplexer, expect reduced capability.

---

## Limitations

### What FrankenTUI Doesn’t Do (Yet)

- **Stable public API**: APIs are evolving quickly.
- **Full widget ecosystem**: Core widgets exist, but the ecosystem is still growing.
- **Guaranteed behavior on every terminal**: Capability detection is conservative; older terminals may degrade.

### Known Limitations

| Capability | Current State | Planned |
|------------|---------------|---------|
| Stable API | ❌ Not yet | Yes (post‑v1) |
| Full widget ecosystem | ⚠️ Partial | Expanding |
| Formal compatibility matrix | ⚠️ In progress | Yes |

---

## FAQ

### Why “FrankenTUI”?

It’s a modular kernel assembled from focused, composable parts — a deliberate, engineered “monster.”

### Is this a full framework?

Not yet. It’s a kernel plus core widgets. You can build a framework on top, but expect APIs to evolve.

### Does it work on Windows?

Windows support is tracked in `docs/WINDOWS.md` and is still being validated.

### Can I embed it in an existing CLI tool?

Yes. Inline mode is designed for CLI + UI coexistence.

### How do I update snapshot tests?

```bash
BLESS=1 cargo test -p ftui-harness
```

---

## Key Docs

- `docs/operational-playbook.md`
- `docs/risk-register.md`
- `docs/glossary.md`
- `docs/adr/README.md`
- `docs/concepts/screen-modes.md`
- `docs/spec/state-machines.md`
- `docs/telemetry.md`
- `docs/spec/telemetry.md`
- `docs/spec/telemetry-events.md`
- `docs/testing/coverage-matrix.md`
- `docs/testing/coverage-playbook.md`
- `docs/one-writer-rule.md`
- `docs/ansi-reference.md`
- `docs/WINDOWS.md`
- `docs/testing/e2e-playbook.md`

---

## Core Algorithms & Data Structures

FrankenTUI isn't just another widget library—it's built on carefully chosen algorithms and data structures optimized for terminal rendering constraints.

### The Cell: A 16-Byte Cache-Optimized Unit

Every terminal cell is exactly **16 bytes**, fitting 4 cells per 64-byte cache line:

```
┌─────────────────────────────────────────────────────────────────┐
│                        Cell (16 bytes)                          │
├─────────────┬─────────────┬─────────────┬─────────────┬────────┤
│ CellContent │     fg      │     bg      │   attrs     │ link_id│
│  (4 bytes)  │ PackedRgba  │ PackedRgba  │  CellAttrs  │  (2B)  │
│  char/gid   │  (4 bytes)  │  (4 bytes)  │  (2 bytes)  │        │
└─────────────┴─────────────┴─────────────┴─────────────┴────────┘
```

**Why 16 bytes?**
- **Cache efficiency:** 4 cells per cache line means sequential row scans hit L1 cache optimally
- **SIMD comparison:** Single 128-bit comparison via `bits_eq()` for cell equality
- **No heap allocation:** 99% of cells store their character inline; only complex graphemes (emoji, ZWJ sequences) use the grapheme pool

### Block-Based Diff Algorithm

The diff engine processes cells in **4-cell blocks** (64 bytes) for autovectorization:

```
for each row:
  if rows_equal(old[y], new[y]):       ← Fast path: skip unchanged rows
    continue

  for each 4-cell block:
    compare 4 × 128-bit cells          ← SIMD-friendly
    if any changed:
      coalesce into ChangeRun          ← Minimize cursor positioning
```

**Key optimizations:**
- **Row-skip fast path:** Unchanged rows detected with single comparison, no cell iteration
- **Dirty row tracking:** Mathematical invariant ensures only mutated rows are checked
- **Change coalescing:** Adjacent changed cells become single `ChangeRun` (one cursor move vs many)

### Presenter Cost Model

The ANSI presenter dynamically chooses the cheapest cursor positioning strategy:

```rust
// CUP (Cursor Position): CSI {row+1};{col+1}H
fn cup_cost(row, col) → 4 + digits(row+1) + digits(col+1)   // e.g., "\x1b[12;45H" = 9 bytes

// CHA (Column Absolute): CSI {col+1}G
fn cha_cost(col) → 3 + digits(col+1)                        // e.g., "\x1b[45G" = 6 bytes

// Per-row decision: sparse runs vs merged write-through
strategy = argmin(sparse_cost, merged_cost)
```

---

## Bayesian Intelligence Layer

FrankenTUI uses principled statistical methods—not ad-hoc heuristics—for runtime decisions.

### BOCPD: Bayesian Online Change-Point Detection

The resize coalescer uses BOCPD to detect regime changes (steady typing vs burst resizing):

```
Observation Model:
  inter-arrival times ~ Exponential(λ_steady) or Exponential(λ_burst)

Run-Length Posterior:
  P(r_t | x_1:t) with truncation at K=100 for O(K) complexity

Regime Decision:
  P(burst | observations) → coalescing delay selection
```

**Why Bayesian?**
- **No magic thresholds:** Prior beliefs updated with evidence
- **Smooth transitions:** Probability-weighted decisions, not binary switches
- **Principled uncertainty:** Knows when it doesn't know

### E-Process: Anytime-Valid Statistical Testing

Budget decisions and alert thresholds use **e-processes** (betting-based sequential tests):

```
Wealth Process:
  W_t = W_{t-1} × (1 + λ_t(X_t - μ₀))

Guarantee:
  P(∃t: W_t ≥ 1/α) ≤ α under null hypothesis

Key Property:
  Valid at ANY stopping time (not just fixed sample sizes)
```

**Practical benefit:** You can check the e-process after every frame without inflating false positive rates.

### VOI Sampling: Value of Information

The runtime decides when to sample expensive metrics using VOI:

```
Beta Posterior: p ~ Beta(α, β)           ← Violation probability
VOI Score:     variance_reduction × value_scale × boundary_weight
Decision:      sample iff (max_interval exceeded) OR (score ≥ cost)
```

This ensures expensive operations (like full diff computation) only run when the information gain justifies the cost.

---

## Performance Engineering

### Dirty Row Tracking

Every buffer mutation marks its row dirty in O(1):

```rust
fn set(&mut self, x: u16, y: u16, cell: Cell) {
    self.cells[y as usize * self.width as usize + x as usize] = cell;
    self.dirty_rows.set(y as usize, true);  // O(1) bitmap write
}
```

**Invariant:** If `is_row_dirty(y) == false`, row y is guaranteed unchanged since last clear.

**Cost:** O(height) space, <2% runtime overhead, but enables skipping 90%+ of cells in typical frames.

### Grapheme Pooling

Complex graphemes (emoji, ZWJ sequences) are reference-counted in a pool:

```
GraphemeId (4 bytes):
┌────────────────────────────────────────┐
│ [31-25: width] [24-0: pool slot index] │
└────────────────────────────────────────┘

Capacity: 16M slots, display widths 0-127
Lookup:   O(1) via HashMap deduplication
```

**Why pooling?**
- Most cells are ASCII (stored inline, no pool lookup)
- Complex graphemes deduplicated (same emoji = same GraphemeId)
- Width embedded in ID (no pool lookup for width queries)

### Synchronized Output

Frames are wrapped in DEC 2026 sync brackets for atomic display:

```
CSI ? 2026 h    ← Begin synchronized update
[all frame output]
CSI ? 2026 l    ← End synchronized update (terminal displays atomically)
```

**Guarantee:** No partial frames ever visible, eliminating flicker even on slow terminals.

---

## The Elm Architecture in Rust

FrankenTUI implements the **Elm/Bubbletea** architecture with Rust's type system:

### The Model Trait

```rust
pub trait Model: Sized {
    type Message: From<Event> + Send + 'static;

    fn init(&mut self) -> Cmd<Self::Message>;
    fn update(&mut self, msg: Self::Message) -> Cmd<Self::Message>;
    fn view(&self, frame: &mut Frame);
    fn subscriptions(&self) -> Vec<Box<dyn Subscription<Self::Message>>>;
}
```

### Update/View Loop

```
┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐
│  Event  │───▶│ Message │───▶│ Update  │───▶│  View   │
│ (input) │    │ (enum)  │    │ (model) │    │ (frame) │
└─────────┘    └─────────┘    └─────────┘    └─────────┘
                                   │              │
                                   ▼              ▼
                              ┌─────────┐    ┌─────────┐
                              │   Cmd   │    │ Render  │
                              │ (async) │    │ (diff)  │
                              └─────────┘    └─────────┘
```

### Commands & Side Effects

```rust
Cmd::none()                    // No side effect
Cmd::perform(future, mapper)   // Async operation → Message
Cmd::quit()                    // Exit program
Cmd::batch(vec![...])          // Multiple commands
```

### Subscriptions

Declarative, long-running event sources:

```rust
fn subscriptions(&self) -> Vec<Box<dyn Subscription<Message>>> {
    vec![
        tick_every(Duration::from_millis(16)),   // 60fps timer
        file_watcher("/path/to/watch"),          // FS events
    ]
}
```

Subscriptions are automatically started/stopped based on what `subscriptions()` returns each frame.

---

## Safety & Correctness Guarantees

### Zero Unsafe Code Policy

```rust
// ftui-render/src/lib.rs
#![forbid(unsafe_code)]

// ftui-runtime/src/lib.rs
#![forbid(unsafe_code)]

// ftui-layout/src/lib.rs
#![forbid(unsafe_code)]
```

The entire render pipeline, runtime, and layout engine contain **zero `unsafe` blocks**.

### Integer Overflow Protection

All coordinate arithmetic uses saturating or checked operations:

```rust
// Cursor positioning (saturating)
let next_x = current_x.saturating_add(width as u16);

// Bounds checking (checked)
let Some(target_x) = x.checked_add(offset) else { continue };

// Intentional wrapping (PRNG only)
seed.wrapping_mul(6364136223846793005).wrapping_add(1)
```

### Flicker-Free Proof Sketch

The codebase includes formal proof sketches in `no_flicker_proof.rs`:

**Theorem 1 (Sync Bracket Completeness):** Every byte emitted by Presenter is wrapped in DEC 2026 sync brackets.

**Theorem 2 (Diff Completeness):** `BufferDiff::compute(old, new)` produces exactly `{(x,y) | old[x,y] ≠ new[x,y]}`.

**Theorem 3 (Dirty Tracking Soundness):** If any cell in row y was mutated, `is_row_dirty(y) == true`.

**Theorem 4 (Diff-Dirty Equivalence):** `compute()` and `compute_dirty()` produce identical output when dirty invariants hold.

---

## Test Infrastructure

### Property-Based Testing

```rust
#[test]
fn prop_diff_soundness() {
    proptest!(|(
        width in 10u16..200,
        height in 5u16..100,
        change_pct in 0.0f64..1.0
    )| {
        // Generate random buffers with controlled change percentage
        // Verify diff output matches actual differences
    });
}
```

### Snapshot Testing

```bash
# Run tests, auto-update baselines
BLESS=1 cargo test -p ftui-harness

# Snapshots stored as .txt files for easy diff review
tests/snapshots/
├── layout_flex_horizontal.txt
├── layout_grid_spanning.txt
└── widget_table_styled.txt
```

### Formal Verification Patterns

```rust
// Proof by counterexample: if this test fails, the theorem is false
#[test]
fn counterexample_dirty_soundness() {
    let mut buf = Buffer::new(10, 10);
    buf.set(5, 5, Cell::from_char('X'));
    assert!(buf.is_row_dirty(5), "Theorem 3 violated: mutation without dirty flag");
}
```

### Benchmark Suite

```bash
cargo bench -p ftui-render

# Output:
# diff/identical_100x50    time: [1.2 µs]   throughput: [4.2 Mcells/s]
# diff/sparse_5pct_100x50  time: [8.3 µs]   throughput: [602 Kcells/s]
# diff/dense_100x50        time: [45 µs]    throughput: [111 Kcells/s]
```

---

## Runtime Systems

### Resize Coalescing

Rapid resize events (e.g., window drag) are coalesced to prevent render thrashing:

```
Event Stream:    R1 ─ R2 ─ R3 ─ R4 ─ R5 ─ [gap] ─ R6
                 └───────────────────┘           │
                        coalesced               applied
                      (only R5 rendered)

Regimes:
  Steady (200ms delay)  ← Responsive to deliberate resizes
  Burst  (20ms delay)   ← Aggressive coalescing during drag
```

### Budget-Based Degradation

Frame time is regulated with a PID controller:

```
Error:        e_t = target_ms - actual_ms
Control:      u_t = Kp·e_t + Ki·Σe + Kd·Δe
Degradation:  Full → SimpleBorders → NoColors → TextOnly

Gains: Kp=0.5, Ki=0.05, Kd=0.2 (tuned for 16ms / 60fps)
```

When frames exceed budget, the renderer automatically degrades visual fidelity to maintain responsiveness.

### Input Fairness Guard

Prevents render work from starving input processing:

```
Fairness Index: F = (Σx_i)² / (n × Σx_i²)   ← Jain's Fairness Index

Intervention: if input_latency > threshold OR F < 0.8:
  force_resize_coalescer_yield()
```

---

## Widget System

### Core Widgets

| Widget | Description | Key Feature |
|--------|-------------|-------------|
| `Block` | Container with borders/title | 9 border styles, title alignment |
| `Paragraph` | Text with wrapping | Word/char wrap, scroll |
| `List` | Selectable items | Virtualized, custom highlight |
| `Table` | Columnar data | Column constraints, row selection |
| `Input` | Text input | Cursor, selection, history |
| `Textarea` | Multi-line input | Line numbers, syntax hooks |
| `Tabs` | Tab bar | Closeable, reorderable |
| `Progress` | Progress bars | Determinate/indeterminate |
| `Sparkline` | Inline charts | Min/max markers |
| `Tree` | Hierarchical data | Expand/collapse, lazy loading |

### Widget Composition

```rust
// Widgets compose via Frame's render method
fn view(&self, frame: &mut Frame) {
    let chunks = Layout::horizontal([
        Constraint::Percentage(30),
        Constraint::Percentage(70),
    ]).split(frame.area());

    frame.render_widget(sidebar, chunks[0]);
    frame.render_widget(main_content, chunks[1]);
}
```

### Stateful Widgets

```rust
// State lives in your Model, widget borrows it
struct MyModel {
    list_state: ListState,
}

fn view(&self, frame: &mut Frame) {
    frame.render_stateful_widget(
        List::new(items),
        area,
        &mut self.list_state,
    );
}
```

---

## Advanced Features

### Hyperlink Support

```rust
let link_id = frame.link_registry().register("https://example.com");
cell.link_id = link_id;
// Emits OSC 8 hyperlink sequences for supporting terminals
```

### Focus Management

```rust
// Declarative focus graph
focus_manager.register("input1", FocusNode::new());
focus_manager.register("input2", FocusNode::new());
focus_manager.set_next("input1", "input2");  // Tab order

// Navigation
focus_manager.focus_next();  // Tab
focus_manager.focus_prev();  // Shift+Tab
```

### Modal System

```rust
modal_stack.push(ConfirmDialog::new("Delete file?"));
// Modals capture input, render above main content
// Escape or button press pops the stack
```

### Time-Travel Debugging

```rust
// Record frames for debugging
let mut recorder = TimeTravel::new();
recorder.record(frame.clone());

// Replay
recorder.seek(frame_index);
let historical_frame = recorder.current();
```

---

## About Contributions

*About Contributions:* Please don't take this the wrong way, but I do not accept outside contributions for any of my projects. I simply don't have the mental bandwidth to review anything, and it's my name on the thing, so I'm responsible for any problems it causes; thus, the risk-reward is highly asymmetric from my perspective. I'd also have to worry about other "stakeholders," which seems unwise for tools I mostly make for myself for free. Feel free to submit issues, and even PRs if you want to illustrate a proposed fix, but know I won't merge them directly. Instead, I'll have Claude or Codex review submissions via `gh` and independently decide whether and how to address them. Bug reports in particular are welcome. Sorry if this offends, but I want to avoid wasted time and hurt feelings. I understand this isn't in sync with the prevailing open-source ethos that seeks community contributions, but it's the only way I can move at this velocity and keep my sanity.

---

## License

No license file is specified yet. If you plan to use FrankenTUI in production, please open an issue to clarify licensing.
